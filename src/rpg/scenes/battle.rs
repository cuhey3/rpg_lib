use crate::engine::application_types::SceneType::RPGBattle;
use crate::engine::application_types::StateType;
use crate::engine::choice::ChoiceSetting;
use crate::engine::input::Input;
use crate::engine::scene::Scene;
use crate::engine::state::State;
use crate::features::animation::{Animation, AnimationSpan};
use crate::rpg::mechanism::choice_kind::ChoiceKind;
use crate::rpg::mechanism::choice_kind::ChoiceKind::Root;
use crate::svg::element_wrapper::ElementWrapper;
use crate::svg::svg_renderer::{RendererController, SvgRenderer};
use rand::{thread_rng, Rng};
use web_sys::Element;

struct BattleElements {
    max_hp_bar: Element,
    current_hp_bar: Element,
}
pub struct BattleState {
    renderer_controller: RendererController,
    elements: BattleElements,
}

impl BattleState {
    pub fn create_battle_scene(shared_state: &mut State) -> Scene {
        let document = &shared_state.elements.document;
        let elements = BattleElements {
            max_hp_bar: document.query_selector("#max-hp-bar").unwrap().unwrap(),
            current_hp_bar: document.query_selector("#current-hp-bar").unwrap().unwrap(),
        };
        let battle_state = BattleState {
            renderer_controller: RendererController {
                renderers: vec![SvgRenderer::new(Root, "battle".to_string(), 40.0)],
                choice_tree: ChoiceSetting::get_battle_setting().get_battle_choice_tree(),
                confirm_index: None,
            },
            elements,
        };
        let consume_func = battle_state.create_consume_func();
        let init_func = battle_state.create_init_func();
        let scene_type = RPGBattle(battle_state);
        Scene {
            own_element: ElementWrapper::new(
                shared_state
                    .elements
                    .document
                    .get_element_by_id("battle")
                    .unwrap(),
            ),
            scene_type,
            is_partial_scene: false,
            consume_func,
            init_func,
            update_map_func: Scene::create_update_map_func_empty(),
            consume_channel_message_func: Scene::create_consume_channel_message_func_empty(),
        }
    }

    pub fn create_init_func(&self) -> fn(&mut Scene, &mut State) {
        fn init_func(scene: &mut Scene, shared_state: &mut State) {
            scene.show();
            if let Scene {
                scene_type: RPGBattle(battle_state),
                ..
            } = scene
            {
                battle_state.renderer_controller.initial_render();
                if let StateType::RPGShared(rpg_shared_state) = &shared_state.state_type {
                    let character = &rpg_shared_state.characters[0];
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
                    shared_state
                        .interrupt_animations
                        .push(vec![Animation::create_message(
                            "ピエンが現れた！".to_string(),
                        )]);
                }
            }
        }

        init_func
    }
    pub fn create_consume_func(&self) -> fn(&mut Scene, &mut State, Input) {
        fn consume_func(scene: &mut Scene, shared_state: &mut State, input: Input) {
            if let RPGBattle(battle_state) = &mut scene.scene_type {
                let BattleState {
                    renderer_controller,
                    ..
                } = battle_state;
                match input {
                    // 矢印キーは状態に応じてカーソルを動かすのみ
                    Input::ArrowUp | Input::ArrowDown | Input::ArrowRight | Input::ArrowLeft => {
                        renderer_controller.delegate_input(input);
                        return;
                    }
                    _ => {}
                }
                if let Input::Enter = input {
                    // 決定キーが押された場合、まず choice_tree の状態を先に進める
                    renderer_controller.delegate_enter();

                    // 先に進めた choice_tree の状態に応じて、画面を更新
                    // 後続処理がないなら return
                    match renderer_controller.now_choice_kind() {
                        ChoiceKind::Battle => {
                            shared_state.primitives.requested_scene_index = 0;
                            shared_state.interrupt_animations.push(vec![
                                Animation::create_multi_line_messages(vec![
                                    "もう戦えない！".to_owned(),
                                    "".to_owned(),
                                    "目の前が真っ暗になった…".to_owned(),
                                ]),
                                Animation::create_fade_out_in_with_span(
                                    AnimationSpan::FadeOutInLong,
                                ),
                            ]);
                            renderer_controller.close_all();
                            return;
                        }
                        ChoiceKind::Escape => {
                            if thread_rng().gen_bool(0.7_f64) {
                                shared_state.primitives.requested_scene_index -= 1;
                                shared_state.interrupt_animations.push(vec![
                                    Animation::create_message("逃げ出した".to_string()),
                                    Animation::create_fade_out_in(),
                                ]);
                                renderer_controller.close_all();
                            } else {
                                shared_state.interrupt_animations.push(vec![
                                    Animation::create_message("逃げられなかった！".to_string()),
                                ]);
                                renderer_controller.undo_choice_tree();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        consume_func
    }
}
