use crate::engine::SharedState;
use crate::rpg::scene::battle::BattleState;
use crate::rpg::scene::field::FieldState;
use crate::rpg::scene::menu::MenuState;
use crate::rpg::scene::title::TitleState;
use crate::rpg::Character;

pub mod battle;
pub mod field;
pub mod menu;
pub mod title;

pub enum SceneType {
    Title(TitleState),
    Field(FieldState),
    Battle(BattleState),
    Menu(MenuState),
}

pub struct Scene {
    pub element_id: String,
    pub scene_type: SceneType,
    pub consume_func:
        fn(scene: &mut Scene, shared_state: &mut SharedState, &mut Vec<Character>, str: String),
    pub init_func: fn(scene: &mut Scene, shared_state: &mut SharedState, &mut Vec<Character>),
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
