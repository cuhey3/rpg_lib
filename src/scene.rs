use crate::scene::scene_type::SceneType;
use crate::{Character, SharedStatus};

pub mod battle;
pub mod field;
pub mod menu;
pub mod scene_type;
pub mod title;

pub struct Scene {
    pub element_id: String,
    pub scene_type: SceneType,
    pub consume_func:
        fn(scene: &mut Scene, shared_status: &mut SharedStatus, &mut Vec<Character>, str: String),
    pub init_func: fn(scene: &mut Scene, shared_status: &mut SharedStatus, &mut Vec<Character>),
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
