use crate::animation::Animation;
use crate::scene::scene_type::SceneType::Battle;
use crate::scene::Scene;
use crate::{Character, Cursor, ElementWrapper, SharedStatus};
use rand::{thread_rng, Rng};
use wasm_bindgen_test::console_log;
use web_sys::Element;

struct BattleElements {
    command: ElementWrapper,
    max_hp_bar: Element,
    current_hp_bar: Element,
}
pub struct BattleStatus {
    elements: BattleElements,
    command_cursor: Cursor,
}

impl BattleStatus {
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut SharedStatus, &mut Vec<Character>) {
        fn init_func(
            scene: &mut Scene,
            shared_status: &mut SharedStatus,
            characters: &mut Vec<Character>,
        ) {
            let battle_status = match &mut scene.scene_type {
                Battle(battle_status) => battle_status,
                _ => panic!(),
            };
            shared_status.elements.battle_scene.show();
            let character = &characters[0];
            let hp_percentage = character.current_hp as f64 / character.max_hp as f64;
            let max_hp_bar_width: f64 = battle_status
                .elements
                .max_hp_bar
                .get_attribute("width")
                .unwrap()
                .parse()
                .unwrap();
            let current_hp_bar_width = max_hp_bar_width * hp_percentage;
            battle_status
                .elements
                .current_hp_bar
                .set_attribute("width", &*current_hp_bar_width.to_string())
                .unwrap();
            match &mut scene.scene_type {
                Battle(battle_status) => {
                    battle_status.command_cursor.reset();
                    shared_status.has_message = true;
                    shared_status
                        .interrupt_animations
                        .push(vec![Animation::create_message(
                            "ピエンが現れた！".to_string(),
                        )]);
                    battle_status.elements.command.show();
                }
                _ => panic!(),
            }
        }
        init_func
    }
    pub fn create_consume_func(
        &self,
    ) -> fn(&mut Scene, &mut SharedStatus, &mut Vec<Character>, String) {
        fn consume_func(
            scene: &mut Scene,
            shared_status: &mut SharedStatus,
            _: &mut Vec<Character>,
            key: String,
        ) {
            match &mut scene.scene_type {
                Battle(battle_status) => {
                    console_log!("battle consume key: {:?}", key);
                    match key.as_str() {
                        "ArrowUp" | "ArrowDown" => {
                            battle_status.command_cursor.consume(key);
                        }
                        "a" => {
                            if battle_status.command_cursor.choose_index != 1 {
                                return;
                            }
                            if thread_rng().gen_bool(0.7_f64) {
                                shared_status.has_message = true;
                                shared_status.requested_scene_index -= 1;
                                battle_status.elements.command.hide();
                                shared_status.interrupt_animations.push(vec![
                                    Animation::create_message("逃げ出した".to_string()),
                                    Animation::create_fade_out_in(),
                                ]);
                            } else {
                                shared_status.has_message = true;
                                shared_status.interrupt_animations.push(vec![
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
pub fn create_battle_scene(shared_status: &mut SharedStatus) -> Scene {
    let document = &shared_status.elements.document;
    let elements = BattleElements {
        command: ElementWrapper::new(document.query_selector("#battle-command").unwrap().unwrap()),
        max_hp_bar: document.query_selector("#max-hp-bar").unwrap().unwrap(),
        current_hp_bar: document.query_selector("#current-hp-bar").unwrap().unwrap(),
    };
    let battle_status = BattleStatus {
        elements,
        command_cursor: Cursor::new(document, "command-cursor", 2, 40.0),
    };
    let consume_func = battle_status.create_consume_func();
    let init_func = battle_status.create_init_func();
    let scene_type = Battle(battle_status);
    Scene {
        element_id: "battle".to_string(),
        scene_type,
        consume_func,
        init_func,
    }
}
