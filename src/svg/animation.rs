use wasm_bindgen_test::console_log;
use web_sys::Element;

pub struct Animation {
    pub args_i32: Vec<i32>,
    pub block_scene_update: bool,
    pub animation_func: fn(&mut Animation, bool, step: f64) -> bool,
    pub elements: Vec<Element>,
    pub start_step: f64,
    pub messages: Vec<String>,
}

impl Animation {
    pub fn set_step(&mut self, step: f64) {
        if self.start_step == -1.0 {
            self.start_step = step;
        }
    }
    pub fn get_step_gap(&self, step: f64) -> f64 {
        step - self.start_step
    }
    pub fn create_fade_out_in() -> Animation {
        Animation {
            args_i32: vec![],
            messages: vec![],
            block_scene_update: true,
            start_step: -1.0,
            elements: vec![web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .query_selector("#fader rect")
                .unwrap()
                .unwrap()],
            animation_func: |animation, _, step| {
                let half_span = 250.0_f64;
                let gap = animation.get_step_gap(step);
                if gap < half_span {
                    animation
                        .elements
                        .get(0)
                        .unwrap()
                        .set_attribute("fill-opacity", &*(gap / half_span).to_string())
                        .expect("TODO: panic message");
                } else {
                    animation
                        .elements
                        .get(0)
                        .unwrap()
                        .set_attribute(
                            "fill-opacity",
                            &*(1.0 - ((gap - half_span) / half_span)).to_string(),
                        )
                        .unwrap();
                }

                if animation.block_scene_update && animation.start_step + half_span < step {
                    animation.block_scene_update = false;
                }
                if gap > half_span * 2.0 {
                    console_log!("animation end");
                    animation
                        .elements
                        .get(0)
                        .unwrap()
                        .set_attribute("fill-opacity", "0")
                        .unwrap();
                    true
                } else {
                    false
                }
            },
        }
    }
    pub fn create_message(message: String) -> Animation {
        let document = web_sys::window().unwrap().document().unwrap();
        let elements = vec![
            document.get_element_by_id("message").unwrap(),
            document.get_element_by_id("message-1").unwrap(),
            document.get_element_by_id("message-2").unwrap(),
        ];
        Animation {
            args_i32: vec![],
            messages: vec![message, "".to_string()],
            block_scene_update: true,
            start_step: -1.0,
            elements,
            animation_func: |animation, has_message, _| {
                if has_message {
                    animation.elements[0]
                        .set_attribute("display", "block")
                        .unwrap();
                    animation.elements[1].set_inner_html(animation.messages.get(0).unwrap());
                    animation.elements[2].set_inner_html(animation.messages.get(1).unwrap());
                }
                animation.block_scene_update = has_message;
                !has_message
            },
        }
    }

    pub fn create_move(start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> Animation {
        let document = web_sys::window().unwrap().document().unwrap();
        let wrapper_element = document.query_selector("#field-wrapper").unwrap().unwrap();
        let character_cursor_element = document
            .query_selector("#character-cursor")
            .unwrap()
            .unwrap();
        let character_direction_element = document
            .query_selector("#character-direction")
            .unwrap()
            .unwrap();
        Animation {
            args_i32: vec![start_x, start_y, end_x, end_y],
            messages: vec![],
            block_scene_update: true,
            start_step: -1.0,
            elements: vec![
                character_cursor_element,
                character_direction_element,
                wrapper_element,
            ],
            animation_func: |animation, _, step| {
                let scale = 150.0;
                let gap = animation.get_step_gap(step).min(scale);
                let start_x = animation.args_i32[0] as f64;
                let start_y = animation.args_i32[1] as f64;
                let end_x = animation.args_i32[2] as f64;
                let end_y = animation.args_i32[3] as f64;
                let step_x = start_x + ((end_x - start_x) / scale * gap);
                let step_y = start_y + ((end_y - start_y) / scale * gap);
                let character_cursor_element = &animation.elements[0];
                let character_direction_element = &animation.elements[1];
                let wrapper_element = &animation.elements[2];
                character_cursor_element
                    .set_attribute("x", &*step_x.to_string())
                    .unwrap();
                character_direction_element
                    .set_attribute("x", &*step_x.to_string())
                    .unwrap();
                character_cursor_element
                    .set_attribute("y", &*step_y.to_string())
                    .unwrap();
                character_direction_element
                    .set_attribute("y", &*(step_y + 35.0).to_string())
                    .unwrap();
                let wrapper_translate_x = 360.0 - step_x;
                let wrapper_translate_y = 280.0 - step_y;
                wrapper_element
                    .set_attribute(
                        "transform",
                        format!(
                            "translate({}, {})",
                            wrapper_translate_x, wrapper_translate_y
                        )
                        .as_str(),
                    )
                    .unwrap();
                gap >= scale
            },
        }
    }
}
