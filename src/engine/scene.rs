use crate::engine::application_types::SceneType;
use crate::engine::{Input, State};

pub struct Scene {
    pub element_id: String,
    pub scene_type: SceneType,
    pub consume_func: fn(scene: &mut Scene, shared_state: &mut State, input: Input),
    pub init_func: fn(scene: &mut Scene, shared_state: &mut State),
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
}
