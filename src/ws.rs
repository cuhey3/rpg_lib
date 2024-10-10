use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use web_sys::WebSocket;

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
#[wasm_bindgen]
pub struct WebSocketWrapper {
    ws: WebSocket,
    is_opened: bool,
    is_closed: bool,
    pub is_joined: bool,
    messages: Vec<ChannelMessage>,
    #[wasm_bindgen(skip)]
    pub user_name: String,
}

impl WebSocketWrapper {
    pub fn new(user_name: String) -> WebSocketWrapper {
        let ws =
            WebSocket::new("https://rust-server-956911707039.asia-northeast1.run.app/ws").unwrap();
        let websocket_wrapper = WebSocketWrapper {
            ws: ws.clone(),
            is_opened: false,
            is_closed: false,
            is_joined: false,
            messages: vec![],
            user_name: user_name.to_owned(),
        };
        let mut clone = websocket_wrapper.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            let channel_user = ChannelUser {
                user_name: clone.user_name.to_owned(),
                channel_name: "rpg".to_string(),
            };
            clone
                .ws
                .send_with_str(&serde_json::to_string(&channel_user).unwrap())
                .unwrap();
            clone.is_opened = true;
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let mut clone = websocket_wrapper.clone();
        let onclose_callback = Closure::<dyn FnMut()>::new(move || {
            clone.is_closed = true;
        });
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        // let mut clone = websocket_wrapper.clone();
        // let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        //     if let Ok(raw_text) = e.data().dyn_into::<js_sys::JsString>() {
        //         let raw_text = raw_text.as_string().unwrap();
        //         let received_message: ChannelMessage = serde_json::from_str(&raw_text).unwrap();
        //         if let Ok(nested_message) =
        //             serde_json::from_str::<ChannelMessage>(&*received_message.message)
        //         {
        //             if nested_message.user_name != clone.user_name {
        //                 clone.messages.push(nested_message);
        //             }
        //         };
        //     }
        // });
        // ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        // onmessage_callback.forget();
        websocket_wrapper
    }

    pub fn is_ready(&self) -> bool {
        self.is_opened && !self.is_closed
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
        self.is_joined = true;
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
        self.is_joined = false;
    }
    pub fn send_message(&self, message: String) {
        self.ws
            .send_with_str(&serde_json::to_string(&message).unwrap())
            .unwrap();
    }
    // Need to wait for onopen event elsewhere
    pub fn update_web_socket(&mut self, web_socket: WebSocket) {
        self.ws = web_socket;
        let channel_user = ChannelUser {
            user_name: self.user_name.to_owned(),
            channel_name: "rpg".to_string(),
        };
        self.ws
            .send_with_str(&serde_json::to_string(&channel_user).unwrap())
            .unwrap()
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
