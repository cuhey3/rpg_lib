use crate::engine::application_types::StateType;
use crate::svg::animation::Animation;
use crate::svg::element_wrapper::ElementWrapper;
use crate::ws::{ChannelMessage, WebSocketWrapper};
use crate::Position;
use application_types::SceneType::RPGField;
use scene::Scene;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::console_log;
use web_sys::Document;

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
            console_log!("keydown interrupt {:?}", key);
            return;
        }
        let scene_index = self.shared_state.primitives.scene_index;
        let consume_func = self.scenes[scene_index].consume_func;
        console_log!("consume start scene: {:?}", scene_index);
        consume_func(&mut self.scenes[scene_index], &mut self.shared_state, key);
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
        if channel_message.user_name == self.web_socket_wrapper.user_name {
            return;
        }
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
                channel_message.message = message;
                field_state.consume_channel_message(&channel_message, &mut self.shared_state);
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
    pub direction: String,
    pub position_x: i32,
    pub position_y: i32,
    pub map_index: usize,
}

pub struct SharedElements {
    pub message: ElementWrapper,
    pub document: Document,
    pub title_scene: ElementWrapper,
    pub event_scene: ElementWrapper,
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
            event_scene: ElementWrapper::new(document.get_element_by_id("event").unwrap()),
            field_scene: ElementWrapper::new(document.get_element_by_id("field").unwrap()),
            battle_scene: ElementWrapper::new(document.get_element_by_id("battle").unwrap()),
            menu_scene: ElementWrapper::new(document.get_element_by_id("menu").unwrap()),
            document,
        }
    }
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
