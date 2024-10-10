use crate::engine::application_types::SceneType::RPGTitle;
use crate::engine::scene::Scene;
use crate::engine::State;
use crate::rpg::RPGSharedState;
use crate::svg::animation::Animation;
use crate::svg::Cursor;

pub struct TitleState {
    cursor: Cursor,
}

impl TitleState {
    pub fn create_title_scene(shared_state: &mut State) -> Scene {
        let document = &shared_state.elements.document;
        let title_state = TitleState {
            cursor: Cursor::new(document, "title-cursor", 3, 60.0),
        };
        let consume_func = title_state.create_consume_func();
        let init_func = title_state.create_init_func();
        let scene_type = RPGTitle(title_state);
        Scene {
            element_id: "title".to_string(),
            scene_type,
            consume_func,
            init_func,
        }
    }
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut State) {
        fn init_func(scene: &mut Scene, shared_state: &mut State) {
            shared_state.elements.title_scene.show();
            match &mut scene.scene_type {
                RPGTitle(..) => {}
                _ => panic!(),
            }
        }
        init_func
    }
    pub fn create_consume_func(&self) -> fn(&mut Scene, &mut State, String) {
        fn consume_func(scene: &mut Scene, shared_state: &mut State, key: String) {
            match &mut scene.scene_type {
                RPGTitle(title_state) => match key.as_str() {
                    "ArrowUp" | "ArrowDown" => {
                        title_state.cursor.consume(key);
                    }
                    "a" => {
                        if title_state.cursor.choose_index == 2 {
                            return;
                        }
                        shared_state.primitives.requested_scene_index = 1;
                        if title_state.cursor.choose_index == 0 {
                            RPGSharedState::new_game(shared_state);
                        } else {
                            RPGSharedState::load_save_data(shared_state);
                        }
                        shared_state
                            .interrupt_animations
                            .push(vec![Animation::create_fade_out_in()]);
                    }
                    _ => (),
                },
                _ => panic!(),
            }
        }
        consume_func
    }
}
