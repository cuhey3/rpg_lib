use crate::application_types::{ApplicationType, StateType};
use crate::rpg::scene::Scene;
use crate::rpg::scene::SceneType::Field;
use crate::rpg::{Character, SaveData};
use crate::svg::animation::Animation;
use crate::svg::element_wrapper::ElementWrapper;
use crate::ws::{ChannelMessage, PositionMessage, WebSocketWrapper};
use crate::Position;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::console_log;
use web_sys::{Document, WebSocket};

#[wasm_bindgen]
pub struct Engine {
    scenes: Vec<Scene>,
    shared_state: SharedState,
    web_socket_wrapper: WebSocketWrapper,
    application_type: ApplicationType,
    state: State,
}

#[wasm_bindgen]
impl Engine {
    pub(crate) fn new(
        application_type: ApplicationType,
        state: State,
        shared_state: SharedState,
        scenes: Vec<Scene>,
        web_socket_wrapper: WebSocketWrapper,
    ) -> Engine {
        Engine {
            application_type,
            scenes,
            shared_state,
            web_socket_wrapper,
            state,
        }
    }

    pub(crate) fn init(&mut self) {
        let init_func = self.scenes[0].init_func;
        init_func(
            &mut self.scenes[0],
            &mut self.shared_state,
        );
    }

    pub fn set_web_socket_instance(&mut self, web_socket: WebSocket) {
        self.web_socket_wrapper.update_web_socket(web_socket);
    }
    pub fn keydown(&mut self, key: String) {
        if self.shared_state.has_message {
            self.shared_state.elements.message.hide();
            self.shared_state.has_message = false;
            return;
        }
        if self.has_animation_blocking_scene_update() {
            console_log!("keydown interrupt {:?}", key);
            return;
        }
        let scene_index = self.shared_state.scene_index;
        let consume_func = self.scenes[scene_index].consume_func;
        console_log!("consume start scene: {:?}", scene_index);
        consume_func(
            &mut self.scenes[scene_index],
            &mut self.shared_state,
            key,
        );
        while !self.shared_state.to_send_channel_messages.is_empty() {
            let message = self.shared_state.to_send_channel_messages.remove(0);
            self.web_socket_wrapper.send_message(message);
        }
        if !self.has_animation_blocking_scene_update() {
            if self.shared_state.scene_index != self.shared_state.requested_scene_index {
                self.shared_state.scene_index = self.shared_state.requested_scene_index;
                self.on_scene_update()
            }
            if self.shared_state.map_index != self.shared_state.requested_map_index {
                self.shared_state.map_index = self.shared_state.requested_map_index;
                self.on_map_update();
            }
        }
    }
    pub fn animate(&mut self, step: f64) {
        for animation in self.shared_state.interrupt_animations.iter_mut() {
            animation.get_mut(0).unwrap().set_step(step);
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
                self.shared_state.has_message,
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
            if self.shared_state.scene_index != self.shared_state.requested_scene_index {
                self.shared_state.scene_index = self.shared_state.requested_scene_index;
                self.on_scene_update()
            }
            if self.shared_state.map_index != self.shared_state.requested_map_index {
                self.shared_state.map_index = self.shared_state.requested_map_index;
                self.on_map_update();
            }
        }
    }
    fn on_scene_update(&mut self) {
        console_log!("scene_updated {:?}", self.shared_state.scene_index);
        let scene_index = self.shared_state.scene_index;
        if scene_index != 0 && !self.web_socket_wrapper.is_joined {
            self.web_socket_wrapper.join();
        }
        if scene_index == 0 && self.web_socket_wrapper.is_joined {
            self.web_socket_wrapper.left();
        }
        if scene_index != 3 {
            for (index, scene) in self.scenes.iter_mut().enumerate() {
                if index != scene_index {
                    scene.hide();
                }
            }
        }
        let init_func = self.scenes[scene_index].init_func;

        init_func(
            &mut self.scenes[scene_index],
            &mut self.shared_state,
        );
    }

    fn on_map_update(&mut self) {
        let scene = &mut self.scenes[self.shared_state.scene_index];
        match scene.scene_type {
            Field(ref mut field_state) => {
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
    pub fn receive_channel_message(&mut self, message: String) {
        if let Ok(mut channel_message) = serde_json::from_str::<ChannelMessage>(&message) {
            if channel_message.user_name == self.web_socket_wrapper.user_name {
                return;
            }
            for scene in self.scenes.iter_mut() {
                if let Scene {
                    scene_type: Field(field_state),
                    ..
                } = scene
                {
                    let mut message = channel_message.message.to_owned();
                    // TODO
                    // ネストしたJSONの扱い…
                    while let Ok(message_string) = serde_json::from_str::<String>(&message) {
                        message = message_string
                    }
                    channel_message.message = message;
                    field_state.consume_channel_message(&channel_message, &mut self.shared_state);
                }
            }
        };
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

pub struct SharedState {
    pub user_name: String,
    pub scene_index: usize,
    pub requested_scene_index: usize,
    pub map_index: usize,
    pub requested_map_index: usize,
    pub interrupt_animations: Vec<Vec<Animation>>,
    pub has_message: bool,
    pub elements: SharedElements,
    pub treasure_box_opened: Vec<Vec<usize>>,
    pub save_data: SaveData,
    pub online_users: Vec<PositionMessage>,
    pub to_send_channel_messages: Vec<String>,
    pub characters: Vec<Character>,
}

impl SharedState {
    pub fn update_save_data(&mut self) {
        self.save_data
            .update(&mut self.characters, &self.treasure_box_opened, self.map_index);
    }
    pub fn load_save_data(&mut self) {
        self.save_data.load(&mut self.characters, true);
        self.treasure_box_opened = self.save_data.treasure_box_usize.to_vec();
        self.map_index = *self.save_data.map_usize.get(0).unwrap();
        self.requested_map_index = *self.save_data.map_usize.get(0).unwrap();
    }
    pub fn new_game(&mut self) {
        let mut new_save_data = SaveData::empty();
        new_save_data.load(&mut self.characters, false);
        self.treasure_box_opened = new_save_data.treasure_box_usize.to_vec();
        console_log!(
            "new_game map1 {} {}",
            self.map_index,
            new_save_data.map_usize.get(0).unwrap()
        );
        self.map_index = *new_save_data.map_usize.get(0).unwrap();
        self.requested_map_index = *new_save_data.map_usize.get(0).unwrap();
        console_log!(
            "new_game map2 {} {}",
            self.map_index,
            new_save_data.map_usize.get(0).unwrap()
        );
    }
}
pub struct SharedElements {
    pub message: ElementWrapper,
    pub document: Document,
    pub title_scene: ElementWrapper,
    pub field_scene: ElementWrapper,
    pub battle_scene: ElementWrapper,
    pub menu_scene: ElementWrapper,
}

impl SharedElements {
    pub fn new() -> SharedElements {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        SharedElements {
            message: ElementWrapper::new(document.get_element_by_id("message").unwrap()),
            title_scene: ElementWrapper::new(document.get_element_by_id("title").unwrap()),
            field_scene: ElementWrapper::new(document.get_element_by_id("field").unwrap()),
            battle_scene: ElementWrapper::new(document.get_element_by_id("battle").unwrap()),
            menu_scene: ElementWrapper::new(document.get_element_by_id("menu").unwrap()),
            document,
        }
    }
}

pub struct State {
    pub to_send_channel_messages: Vec<String>,
    pub state_type: StateType,
}
