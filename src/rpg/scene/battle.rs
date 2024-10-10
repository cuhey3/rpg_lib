use crate::rpg::scene::Scene;
use crate::rpg::scene::SceneType::Battle;
use crate::rpg::{Character, SharedState};
use crate::svg::animation::Animation;
use crate::svg::element_wrapper::ElementWrapper;
use crate::svg::Cursor;
use rand::{thread_rng, Rng};
use wasm_bindgen_test::console_log;
use web_sys::Element;

struct BattleElements {
    command: ElementWrapper,
    max_hp_bar: Element,
    current_hp_bar: Element,
}
pub struct BattleState {
    elements: BattleElements,
    command_cursor: Cursor,
}

impl BattleState {
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut SharedState, &mut Vec<Character>) {
        fn init_func(
            scene: &mut Scene,
            shared_state: &mut SharedState,
            characters: &mut Vec<Character>,
        ) {
            let battle_state = match &mut scene.scene_type {
                Battle(battle_state) => battle_state,
                _ => panic!(),
            };
            shared_state.elements.battle_scene.show();
            let character = &characters[0];
            let hp_percentage = character.current_hp as f64 / character.max_hp as f64;
            let max_hp_bar_width: f64 = battle_state
                .elements
                .max_hp_bar
                .get_attribute("width")
                .unwrap()
                .parse()
                .unwrap();
            let current_hp_bar_width = max_hp_bar_width * hp_percentage;
            battle_state
                .elements
                .current_hp_bar
                .set_attribute("width", &*current_hp_bar_width.to_string())
                .unwrap();
            match &mut scene.scene_type {
                Battle(battle_state) => {
                    battle_state.command_cursor.reset();
                    shared_state.has_message = true;
                    shared_state
                        .interrupt_animations
                        .push(vec![Animation::create_message(
                            "ピエンが現れた！".to_string(),
                        )]);
                    battle_state.elements.command.show();
                }
                _ => panic!(),
            }
        }
        init_func
    }
    pub fn create_consume_func(
        &self,
    ) -> fn(&mut Scene, &mut SharedState, &mut Vec<Character>, String) {
        fn consume_func(
            scene: &mut Scene,
            shared_state: &mut SharedState,
            _: &mut Vec<Character>,
            key: String,
        ) {
            match &mut scene.scene_type {
                Battle(battle_state) => {
                    console_log!("battle consume key: {:?}", key);
                    match key.as_str() {
                        "ArrowUp" | "ArrowDown" => {
                            battle_state.command_cursor.consume(key);
                        }
                        "a" => {
                            if battle_state.command_cursor.choose_index != 1 {
                                return;
                            }
                            if thread_rng().gen_bool(0.7_f64) {
                                shared_state.has_message = true;
                                shared_state.requested_scene_index -= 1;
                                battle_state.elements.command.hide();
                                shared_state.interrupt_animations.push(vec![
                                    Animation::create_message("逃げ出した".to_string()),
                                    Animation::create_fade_out_in(),
                                ]);
                            } else {
                                shared_state.has_message = true;
                                shared_state.interrupt_animations.push(vec![
                                    Animation::create_message("逃げられなかった！".to_string()),
                                ]);
                            }
                        }
                        _ => (),
                    }
                }
                _ => panic!(),
            }
        }
        consume_func
    }
}
pub fn create_battle_scene(shared_state: &mut SharedState) -> Scene {
    let document = &shared_state.elements.document;
    let elements = BattleElements {
        command: ElementWrapper::new(document.query_selector("#battle-command").unwrap().unwrap()),
        max_hp_bar: document.query_selector("#max-hp-bar").unwrap().unwrap(),
        current_hp_bar: document.query_selector("#current-hp-bar").unwrap().unwrap(),
    };
    let battle_state = BattleState {
        elements,
        command_cursor: Cursor::new(document, "command-cursor", 2, 40.0),
    };
    let consume_func = battle_state.create_consume_func();
    let init_func = battle_state.create_init_func();
    let scene_type = Battle(battle_state);
    Scene {
        element_id: "battle".to_string(),
        scene_type,
        consume_func,
        init_func,
    }
}
