use crate::engine::application_types::SceneType::RPGTitle;
use crate::engine::application_types::StateType::RPGShared;
use crate::engine::input::Input;
use crate::engine::scene::Scene;
use crate::engine::state::State;
use crate::features::animation::Animation;
use crate::rpg::RPGSharedState;
use crate::svg::element_wrapper::ElementWrapper;
use crate::svg::svg_renderer::Cursor;

pub struct TitleState {
    cursor: Cursor,
}

impl TitleState {
    pub fn create_title_scene(shared_state: &mut State) -> Scene {
        let document = &shared_state.elements.document;
        let title_state = TitleState {
            cursor: Cursor::new(document, "title-cursor", 2, 60.0),
        };
        let consume_func = title_state.create_consume_func();
        let init_func = title_state.create_init_func();
        let scene_type = RPGTitle(title_state);
        Scene {
            own_element: ElementWrapper::new(document.get_element_by_id("title").unwrap()),
            scene_type,
            is_partial_scene: false,
            consume_func,
            init_func,
            update_map_func: Scene::create_update_map_func_empty(),
            consume_channel_message_func: Scene::create_consume_channel_message_func_empty(),
        }
    }
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut State) {
        fn init_func(scene: &mut Scene, _: &mut State) {
            scene.show();
        }
        init_func
    }
    pub fn create_consume_func(&self) -> fn(&mut Scene, &mut State, Input) {
        fn consume_func(scene: &mut Scene, shared_state: &mut State, input: Input) {
            match &mut scene.scene_type {
                RPGTitle(title_state) => match input {
                    Input::ArrowUp | Input::ArrowDown => {
                        title_state.cursor.consume(input);
                    }
                    Input::Enter => {
                        if title_state.cursor.chose_index == 2 {
                            return;
                        }
                        shared_state.primitives.requested_scene_index = 1;
                        if title_state.cursor.chose_index == 0 {
                            RPGSharedState::new_game(shared_state);
                        } else {
                            RPGSharedState::load_save_data(shared_state);
                            if let State {
                                state_type: RPGShared(rpg_shared_state),
                                ..
                            } = shared_state
                            {
                                let opening_end_flag =
                                    rpg_shared_state.characters[0].event_flags.get(0);
                                if opening_end_flag.is_some() && *opening_end_flag.unwrap() {
                                    shared_state.primitives.requested_scene_index = 2;
                                }
                            }
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
