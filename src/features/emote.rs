use crate::features::animation::{Animation, AnimationSpan};
use serde::{Deserialize, Serialize};

impl Animation {
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
                        .set_attribute("x", (x + 1.5_f64).to_string().as_str())
                        .unwrap();
                    emote_element
                        .set_attribute("y", (y - 19.5_f64).to_string().as_str())
                        .unwrap();
                    emote_element.set_attribute("font-size", "30").unwrap();
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmoteMessage {
    pub user_name: String,
    pub position_x: i32,
    pub position_y: i32,
    pub map_index: usize,
    pub emote: String,
}
