use crate::engine::application_types::SceneType;
use crate::engine::input::Input;
use crate::engine::state::State;
use crate::features::websocket::ChannelMessage;

pub struct Scene {
    pub element_id: String,
    pub scene_type: SceneType,
    pub is_partial_scene: bool,
    pub consume_func: fn(scene: &mut Scene, shared_state: &mut State, input: Input),
    pub init_func: fn(scene: &mut Scene, shared_state: &mut State),
    pub update_map_func: fn(scene: &mut Scene, shared_state: &mut State),
    pub consume_channel_message_func:
        fn(scene: &mut Scene, shared_state: &mut State, message: &ChannelMessage),
}

impl Scene {
    pub fn hide(&self) {
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&*self.element_id)
            .unwrap();
        element.set_attribute("display", "none").unwrap();
    }
    pub fn create_update_map_func_empty() -> fn(&mut Scene, &mut State) {
        fn update_map_func(_: &mut Scene, _: &mut State) {}
        update_map_func
    }

    pub fn create_consume_channel_message_func_empty() -> fn(&mut Scene, &mut State, &ChannelMessage)
    {
        fn consume_channel_message_func(_: &mut Scene, _: &mut State, _: &ChannelMessage) {}
        consume_channel_message_func
    }
}
