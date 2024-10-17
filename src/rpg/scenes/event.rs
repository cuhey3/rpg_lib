use crate::engine::application_types::SceneType::RPGEvent;
use crate::engine::application_types::StateType::RPGShared;
use crate::engine::input::Input;
use crate::engine::scene::Scene;
use crate::engine::state::State;
use crate::features::animation::Animation;
use wasm_bindgen_test::console_log;

pub struct EventState {}

impl EventState {
    pub fn create_event_scene(_: &mut State) -> Scene {
        let event_state = EventState {};
        let consume_func = event_state.create_consume_func();
        let init_func = event_state.create_init_func();
        let scene_type = RPGEvent(event_state);
        Scene {
            element_id: "event".to_string(),
            scene_type,
            consume_func,
            init_func,
            update_map_func: Scene::create_update_map_func_empty(),
        }
    }
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut State) {
        fn init_func(scene: &mut Scene, shared_state: &mut State) {
            console_log!("init event scene");
            // TODO
            // 自分自身の要素をアクティブにする要素を入れる
            shared_state.elements.event_scene.show();
            match &mut scene.scene_type {
                RPGEvent(..) => {}
                _ => panic!(),
            }
            shared_state.primitives.requested_scene_index = 2;
            shared_state.interrupt_animations.push(vec![
                Animation::create_multi_line_messages(vec![
                    "SVG QUEST へようこそ！".to_string(),
                    "".to_string(),
                    "ここは本来オープニングの画面ですが、".to_string(),
                    "まだ用意がありません。".to_string(),
                    "それではごゆっくりお楽しみください。".to_string(),
                ]),
                Animation::create_fade_out_in(),
            ]);
            if let State {
                state_type: RPGShared(rpg_shared_state),
                ..
            } = shared_state
            {
                let opening_end_flag = rpg_shared_state.characters[0].event_flags.get(0);
                if opening_end_flag.is_some() {
                    rpg_shared_state.characters[0].event_flags[0] = true;
                } else {
                    rpg_shared_state.characters[0].event_flags.push(true);
                }
            }
        }
        init_func
    }
    pub fn create_consume_func(&self) -> fn(&mut Scene, &mut State, Input) {
        fn consume_func(_: &mut Scene, _: &mut State, _: Input) {}
        consume_func
    }
}
