use crate::animation::Animation;
use crate::scene::scene_type::SceneType::Title;
use crate::scene::Scene;
use crate::{Character, Cursor, SharedStatus};

pub struct TitleStatus {
    cursor: Cursor,
}

impl TitleStatus {
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut SharedStatus, &mut Vec<Character>) {
        fn init_func(scene: &mut Scene, shared_status: &mut SharedStatus, _: &mut Vec<Character>) {
            shared_status.elements.title_scene.show();
            match &mut scene.scene_type {
                Title(..) => {}
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
            characters: &mut Vec<Character>,
            key: String,
        ) {
            match &mut scene.scene_type {
                Title(title_status) => match key.as_str() {
                    "ArrowUp" | "ArrowDown" => {
                        title_status.cursor.consume(key);
                    }
                    "a" => {
                        if title_status.cursor.choose_index == 2 {
                            return;
                        }
                        shared_status.requested_scene_index = 1;
                        if title_status.cursor.choose_index == 0 {
                            shared_status.new_game(characters);
                        } else {
                            shared_status.load_save_data(characters);
                        }
                        shared_status
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
pub fn create_title_scene(shared_status: &mut SharedStatus) -> Scene {
    let document = &shared_status.elements.document;
    let title_status = TitleStatus {
        cursor: Cursor::new(document, "title-cursor", 3, 60.0),
    };
    let consume_func = title_status.create_consume_func();
    let init_func = title_status.create_init_func();
    let scene_type = Title(title_status);
    Scene {
        element_id: "title".to_string(),
        scene_type,
        consume_func,
        init_func,
    }
}
