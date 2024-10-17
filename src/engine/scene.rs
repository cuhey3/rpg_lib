use crate::engine::application_types::SceneType;
use crate::engine::input::Input;
use crate::engine::state::State;

pub struct Scene {
    pub element_id: String,
    pub scene_type: SceneType,
    pub consume_func: fn(scene: &mut Scene, shared_state: &mut State, input: Input),
    pub init_func: fn(scene: &mut Scene, shared_state: &mut State),
    pub update_map_func: fn(scene: &mut Scene, shared_state: &mut State),
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
}
