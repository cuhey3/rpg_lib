use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::console_log;
use web_sys::{MessageEvent, WebSocket};

#[derive(Serialize, Debug)]
struct ChannelUser {
    user_name: String,
    channel_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelMessage {
    pub user_name: String,
    pub message_type: MessageType,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MessageType {
    Message,
    Join,
    Left,
}
#[derive(Clone)]
pub struct WebSocketWrapper {
    ws: WebSocket,
    pub state: Rc<RefCell<WebSocketState>>,
    pub messages: Rc<RefCell<Vec<ChannelMessage>>>,
    pub user_name: String,
}

#[derive(Clone)]
pub struct WebSocketState {
    pub is_opened: bool,
    pub is_closed: bool,
    pub is_joined: bool,
}

impl WebSocketWrapper {
    pub fn new(user_name: String) -> WebSocketWrapper {
        let ws =
            WebSocket::new("https://rust-server-956911707039.asia-northeast1.run.app/ws").unwrap();
        let mut websocket_wrapper = WebSocketWrapper {
            ws,
            state: Rc::new(RefCell::new(WebSocketState {
                is_opened: false,
                is_closed: false,
                is_joined: false,
            })),
            messages: Rc::new(RefCell::new(vec![])),
            user_name: user_name.to_owned(),
        };
        websocket_wrapper.set_callbacks();
        websocket_wrapper
    }

    fn set_callbacks(&mut self) {
        let clone = self.clone();
        let state_clone = self.state.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            console_log!("websocket connection opened.");
            let channel_user = ChannelUser {
                user_name: clone.user_name.to_owned(),
                channel_name: "rpg".to_string(),
            };
            clone
                .ws
                .send_with_str(&serde_json::to_string(&channel_user).unwrap())
                .unwrap();
            let mut state_clone = state_clone.borrow_mut();
            (*state_clone).is_opened = true;
            (*state_clone).is_closed = false;
            (*state_clone).is_joined = false;
        });
        self.ws
            .set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let clone = self.clone();
        let clone_messages = self.messages.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(raw_text) = e.data().dyn_into::<js_sys::JsString>() {
                let raw_text = raw_text.as_string().unwrap();
                let received_message: ChannelMessage = serde_json::from_str(&raw_text).unwrap();
                if received_message.user_name != clone.user_name {
                    let mut clone_messages = clone_messages.borrow_mut();
                    (*clone_messages).push(received_message);
                }
            }
        });
        self.ws
            .set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        let state_clone = self.state.clone();
        let onclose_callback = Closure::<dyn FnMut()>::new(move || {
            console_log!("websocket connection closed.");

            let mut state_clone = state_clone.borrow_mut();
            (*state_clone).is_closed = true;
        });
        self.ws
            .set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
    }

    pub fn is_ready(&self) -> bool {
        self.state.borrow_mut().is_opened && !self.state.borrow_mut().is_closed
    }

    pub fn join(&mut self) {
        let join_message = ChannelMessage {
            user_name: self.user_name.to_owned(),
            message_type: MessageType::Join,
            message: "join".to_string(),
        };
        self.ws
            .send_with_str(&serde_json::to_string(&join_message).unwrap())
            .unwrap();
        self.state.borrow_mut().is_joined = true;
    }
    pub fn left(&mut self) {
        let left_message = ChannelMessage {
            user_name: self.user_name.to_owned(),
            message_type: MessageType::Left,
            message: "left".to_string(),
        };
        self.ws
            .send_with_str(&serde_json::to_string(&left_message).unwrap())
            .unwrap();
        self.state.borrow_mut().is_joined = false;
    }
    pub fn send_message(&mut self, message: String) {
        if !self.is_ready() {
            console_log!("reconnect websocket...");
            self.reconnect();
            return
        }
        self.ws
            .send_with_str(&serde_json::to_string(&message).unwrap())
            .unwrap();
    }

    // Need to wait for onopen event elsewhere
    pub fn reconnect(&mut self) {
        self.ws =
            WebSocket::new("https://rust-server-956911707039.asia-northeast1.run.app/ws").unwrap();
        self.set_callbacks();
    }
}
