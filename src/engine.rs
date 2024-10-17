use crate::engine::application_types::StateType;
use crate::rpg::ChoiceKind;
use crate::rpg::ChoiceKind::*;
use crate::svg::animation::Animation;
use crate::svg::Position;
use crate::svg::SharedElements;
use crate::ws::{ChannelMessage, WebSocketWrapper};
use application_types::SceneType::RPGField;
use scene::Scene;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::console_log;

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
        // 送るべきメッセージが存在していてWebSocketが切れていれば再接続
        if !self.web_socket_wrapper.is_ready()
            && !self.shared_state.to_send_channel_messages.is_empty()
        {
            self.web_socket_wrapper.request_reconnect();
        }

        // WebSocketに届いたメッセージをアプリケーションに処理させる
        while !(*self.web_socket_wrapper.messages.borrow_mut()).is_empty() {
            let mut message = (*self.web_socket_wrapper.messages.borrow_mut()).remove(0);
            self.receive_channel_message(&mut message);
        }

        // 送るべきメッセージが存在していてWebSocket接続が準備できていれば全て送信
        while self.web_socket_wrapper.is_ready()
            && !self.shared_state.to_send_channel_messages.is_empty()
        {
            let message = self.shared_state.to_send_channel_messages.remove(0);
            self.web_socket_wrapper.send_message(message);
        }

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

impl State {
    pub fn send_own_position(&mut self, input: Option<Input>) {
        if let State {
            state_type: StateType::RPGShared(rpg_shared_state),
            primitives,
            to_send_channel_messages,
            ..
        } = self
        {
            let message = PositionMessage {
                user_name: self.user_name.to_owned(),
                direction: if input.is_none() {
                    Input::ArrowDown
                } else {
                    input.unwrap()
                },
                position_x: rpg_shared_state.characters[0].position.x,
                position_y: rpg_shared_state.characters[0].position.y,
                map_index: primitives.requested_map_index,
            };
            to_send_channel_messages.push(serde_json::to_string(&message).unwrap());
        }
    }
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
            "z" => Input::Cancel,
            "ArrowRight" => Input::ArrowRight,
            "ArrowLeft" => Input::ArrowLeft,
            "ArrowUp" => Input::ArrowUp,
            "ArrowDown" => Input::ArrowDown,
            _ => Input::None,
        }
    }
}

pub struct ChoiceTree {
    pub chose_kinds: Vec<ChoiceKind>,
    pub choice_list: Vec<Vec<String>>,
    pub now_choice: Choice,
    pub choice_indexes: Vec<usize>,
    pub root_choice: Choice,
}

impl ChoiceTree {
    pub fn get_now(&self) -> ChoiceKind {
        self.now_choice.own_token.clone()
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
        if let ChoseNth(..) = &self.now_choice.own_token {
            rewritable_index = 0_usize;
        }
        if let Some(branch) = &mut self.now_choice.branch {
            if let Some(choice) = branch.get_mut(rewritable_index) {
                if let ChoseNth(token, ..) = &choice.own_token {
                    choice.own_token = ChoseNth(token.clone(), Some(index));
                }
                self.chose_kinds.push(self.now_choice.own_token.clone());
                self.now_choice = choice.clone();
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    pub fn undo(&mut self) {
        let indexes_len = self.choice_indexes.len();
        if indexes_len == 0 {
            return;
        }
        self.choice_indexes.remove(indexes_len - 1);
        let copied_choice_indexes = self.choice_indexes.clone();
        self.reset();
        for index in copied_choice_indexes.iter() {
            self.choose(*index);
        }
    }

    fn reset(&mut self) {
        self.now_choice = self.root_choice.clone();
        self.choice_indexes = vec![];
        self.chose_kinds = vec![];
        self.choice_list = vec![];
    }
}

pub struct ChoiceSetting {
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug)]
pub struct Choice {
    pub own_token: ChoiceKind,
    pub label: String,
    pub branch_description: Option<String>,
    pub branch: Option<Vec<Choice>>,
}

impl Choice {
    pub fn confirm_choice() -> Choice {
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
    pub fn no_choice_from(own_token: ChoiceKind) -> Choice {
        Choice {
            label: own_token.get_choice_string(),
            own_token,
            branch_description: None,
            branch: None,
        }
    }
    fn no_choice_from_with_label(own_token: ChoiceKind, label: String) -> Choice {
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
