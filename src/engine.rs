use crate::engine::application_types::StateType;
use crate::engine::ChoiceToken::*;
use crate::svg::animation::Animation;
use crate::svg::{Cursor, CursorType, SharedElements};
use crate::ws::{ChannelMessage, WebSocketWrapper};
use crate::Position;
use application_types::SceneType::RPGField;
use scene::Scene;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::console_log;
use web_sys::Element;

pub mod application_types;
pub mod scene;

#[wasm_bindgen]
pub struct Engine {
    pub(crate) scenes: Vec<Scene>,
    pub(crate) web_socket_wrapper: WebSocketWrapper,
    pub(crate) shared_state: State,
}

#[wasm_bindgen]
impl Engine {
    pub(crate) fn new(
        shared_state: State,
        scenes: Vec<Scene>,
        web_socket_wrapper: WebSocketWrapper,
    ) -> Engine {
        Engine {
            scenes,
            shared_state,
            web_socket_wrapper,
        }
    }

    pub fn keydown(&mut self, key: String) {
        let input = Input::from(key);
        if self.shared_state.references.borrow_mut().has_message {
            if !self
                .shared_state
                .references
                .borrow_mut()
                .has_continuous_message
            {
                self.shared_state.elements.message.hide();
            }
            (*self.shared_state.references.borrow_mut()).has_message = false;
            return;
        }
        if self.has_animation_blocking_scene_update() {
            console_log!("keydown interrupt {:?}", input);
            return;
        }
        let scene_index = self.shared_state.primitives.scene_index;
        let consume_func = self.scenes[scene_index].consume_func;
        console_log!("consume start scene: {:?}", scene_index);
        consume_func(&mut self.scenes[scene_index], &mut self.shared_state, input);
        while !self.shared_state.to_send_channel_messages.is_empty() {
            let message = self.shared_state.to_send_channel_messages.remove(0);
            self.web_socket_wrapper.send_message(message);
        }
        if !self.has_animation_blocking_scene_update() {
            if self.shared_state.primitives.scene_index
                != self.shared_state.primitives.requested_scene_index
            {
                self.shared_state.primitives.scene_index =
                    self.shared_state.primitives.requested_scene_index;
                self.on_scene_update()
            }
            if self.shared_state.primitives.map_index
                != self.shared_state.primitives.requested_map_index
            {
                self.shared_state.primitives.map_index =
                    self.shared_state.primitives.requested_map_index;
                self.on_map_update();
            }
        }
    }
    pub fn animate(&mut self, step: f64) {
        while !(*self.web_socket_wrapper.messages.borrow_mut()).is_empty() {
            let mut message = (*self.web_socket_wrapper.messages.borrow_mut()).remove(0);
            self.receive_channel_message(&mut message);
        }
        // for animation in self.shared_state.interrupt_animations.iter_mut() {
        //     animation.get_mut(0).unwrap().set_step(step);
        // }

        let mut to_delete_indexes = vec![];
        for (index, animation) in self
            .shared_state
            .interrupt_animations
            .iter_mut()
            .enumerate()
        {
            let func = animation.get(0).unwrap().animation_func;
            let result = func(
                animation.get_mut(0).unwrap(),
                self.shared_state.references.clone(),
                step,
            );
            if result {
                to_delete_indexes.push(index)
            }
        }

        to_delete_indexes.reverse();
        for index in to_delete_indexes.iter() {
            let at_animations = self
                .shared_state
                .interrupt_animations
                .get_mut(*index)
                .unwrap();
            at_animations.remove(0);
            if at_animations.is_empty() {
                self.shared_state.interrupt_animations.remove(*index);
            }
        }

        if self
            .shared_state
            .interrupt_animations
            .iter()
            .filter(|animation| animation.get(0).unwrap().block_scene_update)
            .collect::<Vec<&Vec<Animation>>>()
            .len()
            == 0
        {
            if self.shared_state.primitives.scene_index
                != self.shared_state.primitives.requested_scene_index
            {
                self.shared_state.primitives.scene_index =
                    self.shared_state.primitives.requested_scene_index;
                self.on_scene_update()
            }
            if self.shared_state.primitives.map_index
                != self.shared_state.primitives.requested_map_index
            {
                self.shared_state.primitives.map_index =
                    self.shared_state.primitives.requested_map_index;
                self.on_map_update();
            }
        }
    }
    fn on_scene_update(&mut self) {
        console_log!(
            "scene_updated {:?}",
            self.shared_state.primitives.scene_index
        );
        let scene_index = self.shared_state.primitives.scene_index;
        if scene_index != 0 && !self.web_socket_wrapper.state.borrow_mut().is_joined {
            self.web_socket_wrapper.join();
        }
        if scene_index == 0 && self.web_socket_wrapper.state.borrow_mut().is_joined {
            self.web_socket_wrapper.left();
        }
        // TODO
        // メニュー画面だけ特別扱いしていて、シーンを追加するとメニューのインデックスがずれる
        if scene_index != 4 {
            for (index, scene) in self.scenes.iter_mut().enumerate() {
                if index != scene_index {
                    scene.hide();
                }
            }
        }
        let init_func = self.scenes[scene_index].init_func;

        init_func(&mut self.scenes[scene_index], &mut self.shared_state);
    }

    fn on_map_update(&mut self) {
        let scene = &mut self.scenes[self.shared_state.primitives.scene_index];
        // TODO
        // RPGに依存しないコードにしないといけない
        match scene.scene_type {
            RPGField(ref mut field_state) => {
                field_state.update_map(&mut self.shared_state);
            }
            _ => {}
        }
    }

    fn has_animation_blocking_scene_update(&self) -> bool {
        self.shared_state
            .interrupt_animations
            .iter()
            .find(|animation| animation.get(0).unwrap().block_scene_update)
            .is_some()
    }

    fn receive_channel_message(&mut self, channel_message: &mut ChannelMessage) {
        // if channel_message.user_name == self.web_socket_wrapper.user_name {
        //     return;
        // }
        for scene in self.scenes.iter_mut() {
            if let Scene {
                scene_type: RPGField(field_state),
                ..
            } = scene
            {
                let mut message = channel_message.message.to_owned();
                // TODO
                // ネストしたJSONの扱い…
                while let Ok(message_string) = serde_json::from_str::<String>(&message) {
                    message = message_string
                }
                if let Ok(emote_message) = serde_json::from_str::<EmoteMessage>(&message) {
                    field_state.consume_emote_message(emote_message, &mut self.shared_state);
                    return;
                }
                if channel_message.user_name != self.web_socket_wrapper.user_name {
                    channel_message.message = message;
                    field_state.consume_channel_message(&channel_message, &mut self.shared_state);
                }
            }
        }
    }
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position { x, y }
    }
    pub fn new_vec(args: Vec<[i32; 2]>) -> Vec<Position> {
        args.iter()
            .map(|arg| Position::new(arg[0], arg[1]))
            .collect()
    }
    pub fn new_area(areas: Vec<[i32; 4]>) -> Vec<Position> {
        let mut result = vec![];
        for area in areas.iter() {
            let [start_x, start_y, end_x, end_y] = *area;
            let step_x = (end_x - start_x) / 40;
            let step_y = (end_y - start_y) / 40;
            for y in 0..step_y + 1 {
                if y == 0 {
                    for x in 0..step_x + 1 {
                        result.push(Position::new(start_x + x * 40, start_y + y * 40))
                    }
                } else if y == end_y {
                } else {
                    result.push(Position::new(start_x, start_y + y * 40));
                    result.push(Position::new(end_x, start_y + y * 40));
                }
            }
        }
        result
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PositionMessage {
    pub user_name: String,
    pub direction: Input,
    pub position_x: i32,
    pub position_y: i32,
    pub map_index: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmoteMessage {
    pub user_name: String,
    pub position_x: i32,
    pub position_y: i32,
    pub map_index: usize,
    pub emote: String,
}

pub struct Primitives {
    pub scene_index: usize,
    pub requested_scene_index: usize,
    pub map_index: usize,
    pub requested_map_index: usize,
}

pub struct References {
    pub has_message: bool,
    pub has_continuous_message: bool,
}

pub struct State {
    pub user_name: String,
    pub to_send_channel_messages: Vec<String>,
    pub state_type: StateType,
    pub elements: SharedElements,
    pub interrupt_animations: Vec<Vec<Animation>>,
    pub primitives: Primitives,
    pub references: Rc<RefCell<References>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    Enter,
    Cancel,
    Context,
    ArrowRight,
    ArrowLeft,
    ArrowUp,
    ArrowDown,
    None,
}

impl Input {
    pub fn from(key: String) -> Input {
        match key.as_str() {
            "a" => Input::Enter,
            "Escape" => Input::Cancel,
            "ArrowRight" => Input::ArrowRight,
            "ArrowLeft" => Input::ArrowLeft,
            "ArrowUp" => Input::ArrowUp,
            "ArrowDown" => Input::ArrowDown,
            _ => Input::None,
        }
    }
}

pub struct ChoiceInstance {
    pub choice_tokens: Vec<ChoiceToken>,
    choice_list: Vec<Vec<String>>,
    pub now_choice: Choice,
    choice_indexes: Vec<usize>,
    root_choice: Choice,
}

impl ChoiceInstance {
    fn open() {}

    fn close() {}

    fn is_complete(&self) -> bool {
        if let Undo = self.now_choice.own_token {
            false
        } else {
            self.now_choice.branch.is_none()
        }
    }

    pub fn get_now(&self) -> ChoiceToken {
        self.now_choice.own_token.clone()
    }
    fn get_tokens(&self) -> Vec<ChoiceToken> {
        vec![]
    }

    pub fn choose(&mut self, index: usize) {
        self.choice_indexes.push(index);
        let mut rewritable_index = index;
        if let ItemInventory = &self.now_choice.own_token {
            rewritable_index = 0_usize;
        }
        if let Emote = &self.now_choice.own_token {
            rewritable_index = 0_usize;
        }
        if let NthChoose(..) = &self.now_choice.own_token {
            rewritable_index = 0_usize;
        }
        if let Some(branch) = &mut self.now_choice.branch {
            if let Some(choice) = branch.get_mut(rewritable_index) {
                if let NthChoose(token, ..) = &choice.own_token {
                    choice.own_token = NthChoose(token.clone(), Some(index));
                }
                self.choice_tokens.push(self.now_choice.own_token.clone());
                self.now_choice = choice.clone();
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }
    fn fill_choice_list(&mut self, list: Vec<Vec<String>>) {
        self.choice_list = list
    }

    pub fn undo(&mut self) {
        let indexes_len = self.choice_indexes.len();
        if indexes_len == 0 {
            todo!()
        }
        self.choice_indexes.remove(indexes_len - 1);
        let copied_choice_indexes = self.choice_indexes.clone();
        self.reset();
        for index in copied_choice_indexes.iter() {
            self.choose(*index);
        }
    }

    fn decide(&mut self) {
        self.undo();
        self.now_choice.branch = None;
    }

    fn reset(&mut self) {
        self.now_choice = self.root_choice.clone();
        self.choice_indexes = vec![];
        self.choice_tokens = vec![];
        self.choice_list = vec![];
    }
}

pub struct ChoiceSetting {
    choices: Vec<Choice>,
}

#[derive(Clone, Debug)]
pub struct Choice {
    pub own_token: ChoiceToken,
    pub label: String,
    pub branch_description: Option<String>,
    pub branch: Option<Vec<Choice>>,
}

impl Choice {
    fn confirm_choice() -> Choice {
        Choice {
            label: Confirm.get_choice_string(),
            own_token: Confirm,
            branch_description: None,
            branch: Some(vec![
                Choice::no_choice_from_with_label(Decide, "はい".to_string()),
                Choice::no_choice_from_with_label(Undo, "いいえ".to_string()),
            ]),
        }
    }
    fn no_choice_from(own_token: ChoiceToken) -> Choice {
        Choice {
            label: own_token.get_choice_string(),
            own_token,
            branch_description: None,
            branch: None,
        }
    }
    fn no_choice_from_with_label(own_token: ChoiceToken, label: String) -> Choice {
        Choice {
            label,
            own_token,
            branch_description: None,
            branch: None,
        }
    }
    pub fn label_or_token_string(&self) -> String {
        if self.label.is_empty() {
            self.own_token.get_choice_string()
        } else {
            self.label.to_owned()
        }
    }
    pub fn get_branch_labels(&self) -> Vec<String> {
        if let Some(branch) = &self.branch {
            branch
                .iter()
                .map(|choice| choice.label_or_token_string())
                .collect()
        } else {
            vec![]
        }
    }
}
#[derive(Clone, Debug)]
pub enum ChoiceToken {
    Menu,
    UseItem,
    DropItem,
    Yes,
    No,
    Battle,
    Escape,
    Special,
    ItemInventory,
    Spell,
    Equip,
    Save,
    Title,
    Close,
    Emote,
    SendEmote,
    Chat,
    Nth(String),
    NthChoose(String, Option<usize>),
    ItemOperation,
    Confirm,
    Decide,
    Undo,
    Root,
}

impl ChoiceToken {
    fn from_string(token: String) -> ChoiceToken {
        match token.as_str() {
            "Item" => ItemInventory,
            _ => panic!(),
        }
    }
    fn get_choice_string(&self) -> String {
        match self {
            Root => "",
            Menu => "",
            UseItem => "つかう",
            DropItem => "すてる",
            Yes => "はい",
            No => "いいえ",
            Battle => "たたかう",
            Escape => "にげる",
            Special => "とくぎ",
            ItemInventory => "どうぐ",
            Spell => "じゅもん",
            Equip => "そうび",
            Save => "セーブ",
            Title => "タイトル",
            Close => "とじる",
            Emote => "エモート",
            SendEmote => "",
            Chat => "チャット",
            Confirm => "",
            Undo => "",
            Decide => "",
            Nth(..) => "",
            NthChoose(..) => "",
            ItemOperation => "",
        }
        .to_string()
    }
}

impl ChoiceSetting {
    fn new() -> ChoiceSetting {
        ChoiceSetting { choices: vec![] }
    }
    fn add_choices(&mut self, choice: &mut Vec<Choice>) -> &mut ChoiceSetting {
        self.choices.append(choice);
        self
    }
    pub fn get_instance(&self) -> ChoiceInstance {
        let root_choice = Choice {
            own_token: Root,
            label: Root.get_choice_string(),
            branch_description: None,
            branch: Some(self.choices.clone()),
        };
        ChoiceInstance {
            choice_tokens: vec![],
            choice_list: vec![],
            choice_indexes: vec![],
            now_choice: root_choice.clone(),
            root_choice,
        }
    }

    pub fn get_menu_instance(&self) -> ChoiceInstance {
        let root_choice = Choice {
            own_token: Menu,
            label: Menu.get_choice_string(),
            branch_description: None,
            branch: Some(self.choices.clone()),
        };
        ChoiceInstance {
            choice_tokens: vec![],
            choice_list: vec![],
            choice_indexes: vec![],
            now_choice: root_choice.clone(),
            root_choice,
        }
    }
    pub fn get_menu_setting() -> ChoiceSetting {
        let use_choice = Choice::no_choice_from(UseItem);
        let drop_choice = Choice {
            own_token: DropItem,
            label: DropItem.get_choice_string(),
            branch_description: Some("本当に捨てますか？".to_string()),
            branch: Some(vec![Choice::confirm_choice()]),
        };
        let mut setting = ChoiceSetting::new();
        setting.add_choices(&mut vec![
            Choice {
                own_token: ItemInventory,
                label: "".to_string(),
                branch_description: None,
                branch: Some(vec![Choice {
                    own_token: NthChoose("Item".to_string(), None),
                    label: "".to_string(),
                    branch_description: None,
                    branch: Some(vec![Choice {
                        own_token: ItemOperation,
                        label: "".to_string(),
                        branch_description: None,
                        branch: Some(vec![use_choice.clone(), drop_choice.clone()]),
                    }]),
                }]),
            },
            Choice::no_choice_from(Equip),
            Choice {
                own_token: Emote,
                label: "".to_string(),
                branch_description: None,
                branch: Some(vec![Choice {
                    own_token: NthChoose("Emote".to_string(), None),
                    label: "".to_string(),
                    branch_description: None,
                    branch: Some(vec![Choice::no_choice_from(SendEmote)]),
                }]),
            },
            Choice::no_choice_from(Chat),
            Choice {
                own_token: Save,
                label: "".to_string(),
                branch_description: Some("セーブを上書きします。よろしいですか？".to_string()),
                branch: Some(vec![Choice::confirm_choice()]),
            },
            Choice {
                own_token: Title,
                label: "".to_string(),
                branch_description: Some("タイトルに戻ります。よろしいですか？".to_string()),
                branch: Some(vec![Choice::confirm_choice()]),
            },
            Choice::no_choice_from(Close),
        ]);
        setting
    }
}

pub struct SvgRenderer {
    target_part_name: String,
    wrapper_element: Option<Element>,
    item_element: Option<Element>,
    message_wrapper_element: Option<Element>,
    message_element: Option<Element>,
    pub cursor: Cursor,
    step_length: f64,
    item_labels: Vec<String>,
    item_x: f64,
    item_y: f64,
}
impl SvgRenderer {
    pub fn new(target_part_name: String, step_length: f64) -> SvgRenderer {
        SvgRenderer {
            target_part_name,
            wrapper_element: None,
            item_element: None,
            message_wrapper_element: None,
            message_element: None,
            cursor: Cursor::empty(),
            step_length,
            item_labels: vec![],
            item_x: 0.0,
            item_y: 0.0,
        }
    }
    pub fn load(&mut self) {
        self.load_wrapper_element();
        self.load_item_element();
        self.load_cursor();
        self.load_message_wrapper_element();
        self.load_message_element();
    }
    pub fn load_wrapper_element(&mut self) {
        self.wrapper_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_wrapper_id())
    }
    pub fn get_wrapper_id(&self) -> String {
        format!("render-{}-wrapper", self.target_part_name)
    }

    pub fn load_cursor(&mut self) {
        let cursor_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_cursor_id())
            .unwrap();
        self.cursor = Cursor::new_with_element(cursor_element, self.step_length);
    }

    pub fn get_cursor_id(&self) -> String {
        format!("render-{}-cursor", self.target_part_name)
    }

    pub fn load_item_element(&mut self) {
        self.item_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_item_id());
        if let Some(element) = &self.item_element {
            self.item_x = element.get_attribute("x").unwrap().parse().unwrap();
            self.item_y = element.get_attribute("y").unwrap().parse().unwrap();
        }
    }
    pub fn get_item_id(&self) -> String {
        format!("render-{}-item", self.target_part_name)
    }
    pub fn load_message_wrapper_element(&mut self) {
        self.message_wrapper_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_message_wrapper_id())
    }
    pub fn get_message_wrapper_id(&self) -> String {
        format!("render-{}-message-wrapper", self.target_part_name)
    }

    pub fn load_message_element(&mut self) {
        self.message_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_message_id())
    }
    pub fn get_message_id(&self) -> String {
        format!("render-{}-message", self.target_part_name)
    }

    pub fn get_rendered_id(&self) -> String {
        format!("render-{}-rendered", self.target_part_name)
    }

    pub fn render(&mut self, labels: Vec<String>, description: &str) {
        self.item_labels = labels;
        let document = web_sys::window().unwrap().document().unwrap();
        if let Some(to_remove) = document.get_element_by_id(self.get_rendered_id().as_str()) {
            to_remove.remove();
        }
        let group_element = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "g")
            .unwrap();
        group_element
            .set_attribute("id", self.get_rendered_id().as_str())
            .unwrap();
        if let Some(wrapper_element) = &self.wrapper_element {
            wrapper_element.append_child(&*group_element).unwrap();
            wrapper_element.set_attribute("display", "block").unwrap();
        }

        for (index, label) in self.item_labels.iter().enumerate() {
            if let Some(item_element) = &self.item_element {
                let node = item_element.clone_node().unwrap();
                let empty_element = document
                    .create_element_ns(Some("http://www.w3.org/2000/svg"), "text")
                    .unwrap();
                node.append_child(&*empty_element).unwrap();
                let element = empty_element.parent_element().unwrap();
                element.set_inner_html(label);
                match self.cursor.cursor_type {
                    CursorType::Default => {
                        element
                            .set_attribute("x", &*self.item_x.to_string())
                            .unwrap();
                        element
                            .set_attribute(
                                "y",
                                &*(self.item_y + index as f64 * self.step_length).to_string(),
                            )
                            .unwrap();
                    }
                    CursorType::Box => {
                        let (x, y) = self.cursor.index_to_box_x_box_y(index);
                        element
                            .set_attribute(
                                "x",
                                &*(self.item_x + x as f64 * self.step_length).to_string(),
                            )
                            .unwrap();
                        element
                            .set_attribute(
                                "y",
                                &*(self.item_y + y as f64 * self.step_length).to_string(),
                            )
                            .unwrap();
                    }
                    _ => {}
                }
                element.set_attribute("display", "block").unwrap();
                group_element.append_child(&*element).unwrap();
            }
        }
        self.cursor.reset();
        self.cursor
            .element
            .set_attribute("display", "block")
            .unwrap();
        if let Some(element) = &self.message_wrapper_element {
            let display = if description.is_empty() {
                "none"
            } else {
                "block"
            };
            element.set_attribute("display", display).unwrap();
        }
        if let Some(element) = &self.message_element {
            if !description.is_empty() {
                element.set_inner_html(description);
            }
        }
    }
    pub fn hide(&self) {
        if let Some(element) = &self.wrapper_element {
            element.set_attribute("display", "none").unwrap();
        }
    }
}
