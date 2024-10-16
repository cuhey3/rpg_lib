use crate::engine::{EmoteMessage, References};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::console_log;
use web_sys::Element;

pub struct Animation {
    pub args_i32: Vec<i32>,
    pub block_scene_update: bool,
    pub animation_func: fn(&mut Animation, Rc<RefCell<References>>, step: f64) -> bool,
    pub elements: Vec<Element>,
    pub start_step: f64,
    pub span: AnimationSpan,
    pub messages: Vec<String>,
}

impl Animation {
    pub fn init_step(&mut self, step: f64) {
        if self.start_step == -1.0 {
            self.start_step = step;
        }
    }
    pub fn get_step_gap(&self, step: f64) -> f64 {
        step - self.start_step
    }

    pub fn always_blink() -> Animation {
        let node_list = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .query_selector_all(".always-blink")
            .unwrap();
        let mut elements = vec![];
        for n in 0..node_list.length() {
            let element = node_list.item(n).unwrap().dyn_into::<Element>().unwrap();
            elements.push(element);
        }
        Animation {
            args_i32: vec![],
            messages: vec![],
            block_scene_update: false,
            start_step: -1.0,
            elements,
            span: AnimationSpan::None,
            animation_func: move |animation, _, step| {
                animation.init_step(step);
                let gap = animation.get_step_gap(step);
                let gap_sin = (gap / 250_f64).sin() / 2_f64 + 0.5;
                for element in animation.elements.iter() {
                    element
                        .set_attribute("fill-opacity", &*gap_sin.to_string())
                        .unwrap();
                }

                false
            },
        }
    }

    pub fn show_emote(message: EmoteMessage, own_emote: bool) -> Animation {
        Animation {
            args_i32: vec![if own_emote { 1 } else { -1 }],
            messages: vec![message.user_name, message.emote],
            block_scene_update: false,
            start_step: -1.0,
            elements: vec![],
            span: AnimationSpan::EmoteDefault,
            animation_func: |animation, _, step| {
                let own_emote = animation.args_i32[0] == 1;
                let document = web_sys::window().unwrap().document().unwrap();
                let another_emote_selector = format!(".emote.user-name-{}", animation.messages[0]);
                let selector_str = if own_emote {
                    ".emote.character"
                } else {
                    another_emote_selector.as_str()
                };
                if let Some(element) = document.query_selector(selector_str).unwrap() {
                    element.remove();
                    if own_emote {
                        document
                            .query_selector(".emote-background.character")
                            .unwrap()
                            .unwrap()
                            .remove();
                        document
                            .query_selector(".emote-background-arrow.character")
                            .unwrap()
                            .unwrap()
                            .remove();
                    } else {
                        document
                            .query_selector(
                                format!(".emote-background.user-name-{}", animation.messages[0])
                                    .as_str(),
                            )
                            .unwrap()
                            .unwrap()
                            .remove();
                        document
                            .query_selector(
                                format!(
                                    ".emote-background-arrow.user-name-{}",
                                    animation.messages[0]
                                )
                                .as_str(),
                            )
                            .unwrap()
                            .unwrap()
                            .remove();
                    }
                };
                animation.init_step(step);
                let gap = animation.get_step_gap(step);
                let span = animation.span.clone() as i32 as f64;
                if gap > span {
                    return true;
                }
                let another_rect_selector =
                    format!(".online-user.user-name-{}", animation.messages[0]);
                if let Some(element) = document
                    .query_selector(if own_emote {
                        "rect.character"
                    } else {
                        another_rect_selector.as_str()
                    })
                    .unwrap()
                {
                    let x: f64 = element.get_attribute("x").unwrap().parse().unwrap();
                    let y: f64 = element.get_attribute("y").unwrap().parse().unwrap();
                    let emote_element = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .create_element_ns(Some("http://www.w3.org/2000/svg"), "text")
                        .unwrap();
                    emote_element.set_inner_html(animation.messages[1].as_str());
                    emote_element
                        .set_attribute("x", (x + 3_f64).to_string().as_str())
                        .unwrap();
                    emote_element
                        .set_attribute("y", (y - 17_f64).to_string().as_str())
                        .unwrap();
                    emote_element.set_attribute("font-size", "35").unwrap();
                    let emote_background = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .create_element_ns(Some("http://www.w3.org/2000/svg"), "rect")
                        .unwrap();
                    emote_background
                        .set_attribute("x", x.to_string().as_str())
                        .unwrap();
                    emote_background
                        .set_attribute("y", (y - 50_f64).to_string().as_str())
                        .unwrap();
                    emote_background.set_attribute("rx", "3").unwrap();
                    emote_background.set_attribute("fill", "white").unwrap();
                    emote_background.set_attribute("width", "40").unwrap();
                    emote_background.set_attribute("height", "39").unwrap();
                    let emote_background_arrow = web_sys::window()
                        .unwrap()
                        .document()
                        .unwrap()
                        .create_element_ns(Some("http://www.w3.org/2000/svg"), "polygon")
                        .unwrap();
                    let point_a = format!("{} {}", x + 34.0, y - 13.0);
                    let point_b = format!("{} {}", x + 25.0, y - 3.0);
                    let point_c = format!("{} {}", x + 25.0, y - 13.0);
                    emote_background_arrow
                        .set_attribute(
                            "points",
                            format!("{}, {}, {}", point_a, point_b, point_c).as_str(),
                        )
                        .unwrap();
                    emote_background_arrow
                        .set_attribute("fill", "white")
                        .unwrap();
                    let parent_element = if own_emote {
                        emote_element
                            .class_list()
                            .add_2("emote", "character")
                            .unwrap();
                        emote_background
                            .class_list()
                            .add_2("emote-background", "character")
                            .unwrap();
                        emote_background_arrow
                            .class_list()
                            .add_2("emote-background-arrow", "character")
                            .unwrap();
                        document.get_element_by_id("field").unwrap()
                    } else {
                        emote_element
                            .class_list()
                            .add_2(
                                "emote",
                                format!("user-name-{}", animation.messages[0]).as_str(),
                            )
                            .unwrap();
                        emote_background
                            .class_list()
                            .add_2(
                                "emote-background",
                                format!("user-name-{}", animation.messages[0]).as_str(),
                            )
                            .unwrap();
                        emote_background_arrow
                            .class_list()
                            .add_2(
                                "emote-background-arrow",
                                format!("user-name-{}", animation.messages[0]).as_str(),
                            )
                            .unwrap();
                        document.get_element_by_id("field-wrapper").unwrap()
                    };
                    if gap > span * 0.95 {
                        let opacity = 1.0 - (gap - span * 0.95) / (span * 0.05);
                        emote_background_arrow
                            .set_attribute("fill-opacity", &*opacity.to_string())
                            .unwrap();
                        emote_background
                            .set_attribute("fill-opacity", &*opacity.to_string())
                            .unwrap();
                        emote_element
                            .set_attribute("fill-opacity", &*opacity.to_string())
                            .unwrap();
                    }
                    parent_element
                        .append_child(&*emote_background_arrow)
                        .unwrap();
                    parent_element.append_child(&*emote_background).unwrap();
                    parent_element.append_child(&*emote_element).unwrap();
                    false
                } else {
                    true
                }
            },
        }
    }
    pub fn create_fade_out_in() -> Animation {
        Animation::create_fade_out_in_with_span(AnimationSpan::FadeOutInDefault)
    }
    pub fn create_fade_out_in_with_span(span: AnimationSpan) -> Animation {
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
            span,
            animation_func: move |animation, _, step| {
                animation.init_step(step);
                let span_f64 = animation.span.clone() as i32 as f64;
                let half_span = span_f64 / 2.0;
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
            messages: vec![message.to_owned()],
            block_scene_update: true,
            start_step: -1.0,
            elements,
            span: AnimationSpan::None,
            animation_func: |animation, references, _| {
                if references.borrow_mut().has_message {
                    return false;
                }
                if animation.messages.is_empty() {
                    animation.block_scene_update = false;
                    return true;
                }
                animation.block_scene_update = true;
                references.borrow_mut().has_message = true;
                animation.elements[0]
                    .set_attribute("display", "block")
                    .unwrap();
                animation.elements[1].set_inner_html(&animation.messages.remove(0));
                animation.elements[2].set_inner_html("");
                return false;
            },
        }
    }
    pub fn create_multi_line_messages(messages: Vec<String>) -> Animation {
        let document = web_sys::window().unwrap().document().unwrap();
        let elements = vec![
            document.get_element_by_id("message").unwrap(),
            document.get_element_by_id("message-1").unwrap(),
            document.get_element_by_id("message-2").unwrap(),
            document
                .get_element_by_id("has-continuous-message")
                .unwrap(),
        ];
        Animation {
            args_i32: vec![],
            messages,
            block_scene_update: true,
            start_step: -1.0,
            elements,
            span: AnimationSpan::None,
            animation_func: |animation, references, _| {
                if references.borrow_mut().has_message {
                    return false;
                }
                if animation.messages.is_empty() {
                    animation.block_scene_update = false;
                    return true;
                }
                animation.block_scene_update = true;
                references.borrow_mut().has_message = true;
                animation.elements[0]
                    .set_attribute("display", "block")
                    .unwrap();
                animation.elements[1].set_inner_html(&animation.messages.remove(0));
                if !animation.messages.is_empty() {
                    animation.elements[2].set_inner_html(&animation.messages.remove(0));
                } else {
                    animation.elements[2].set_inner_html("");
                }
                let has_continuous_message = !animation.messages.is_empty();
                let display = if has_continuous_message {
                    "block"
                } else {
                    "none"
                };
                animation.elements[3]
                    .set_attribute("display", display)
                    .unwrap();
                (*references.borrow_mut()).has_continuous_message = has_continuous_message;
                return false;
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
            span: AnimationSpan::None,
            animation_func: |animation, _, step| {
                animation.init_step(step);
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

#[derive(Clone)]
pub enum AnimationSpan {
    FadeOutInDefault = 500,
    FadeOutInLong = 2000,
    FadeOutInMedium = 1000,
    EmoteDefault = 5000,
    None = 0,
}
