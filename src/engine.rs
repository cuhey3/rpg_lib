use crate::features::animation::Animation;
use crate::features::emote::EmoteMessage;
use crate::features::websocket::{ChannelMessage, WebSocketWrapper};
use application_types::SceneType::RPGField;
use input::Input;
use scene::Scene;
use state::State;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_test::console_log;

pub mod application_types;
pub mod choice;
pub mod input;
pub mod scene;
pub mod state;

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
        let update_map_func = self.scenes[self.shared_state.primitives.scene_index].update_map_func;
        update_map_func(
            &mut self.scenes[self.shared_state.primitives.scene_index],
            &mut self.shared_state,
        );
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
}
