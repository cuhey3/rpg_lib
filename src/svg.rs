use crate::svg::element_wrapper::ElementWrapper;
use web_sys::{Document, Element};

pub mod animation;
pub mod element_wrapper;

pub struct Cursor {
    element: Element,
    pub choose_index: usize,
    choice_length: usize,
    step_height: f64,
    default_y: f64,
}

impl Cursor {
    pub fn new(
        document: &Document,
        cursor_id: &str,
        choice_length: usize,
        step_height: f64,
    ) -> Cursor {
        let element = document.get_element_by_id(cursor_id).unwrap();
        let default_y = element.get_attribute("y").unwrap().parse().unwrap();
        Cursor {
            element,
            choose_index: 0,
            choice_length,
            step_height,
            default_y,
        }
    }
    pub fn update_choice_length(&mut self, choice_length: usize) {
        self.choice_length = choice_length;
        self.choose_index = self.choose_index.min(self.choice_length - 1);
    }
    pub fn reset(&mut self) {
        self.choose_index = 0;
        self.element
            .set_attribute("y", &*self.default_y.to_string())
            .unwrap();
    }
    pub fn consume(&mut self, key: String) {
        let new_index = match key.as_str() {
            "ArrowUp" => (self.choose_index + self.choice_length - 1) % self.choice_length,
            "ArrowDown" => (self.choose_index + 1) % self.choice_length,
            _ => panic!(),
        };
        self.choose_index = new_index;
        let new_y: f64 = self.default_y + new_index as f64 * self.step_height;
        self.element
            .set_attribute("y", new_y.to_string().as_str())
            .unwrap();
    }
}

pub struct SharedElements {
    pub message: ElementWrapper,
    pub document: Document,
    pub title_scene: ElementWrapper,
    pub event_scene: ElementWrapper,
    pub field_scene: ElementWrapper,
    pub battle_scene: ElementWrapper,
    pub menu_scene: ElementWrapper,
}

impl SharedElements {
    pub fn new() -> SharedElements {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        SharedElements {
            message: ElementWrapper::new(document.get_element_by_id("message").unwrap()),
            title_scene: ElementWrapper::new(document.get_element_by_id("title").unwrap()),
            event_scene: ElementWrapper::new(document.get_element_by_id("event").unwrap()),
            field_scene: ElementWrapper::new(document.get_element_by_id("field").unwrap()),
            battle_scene: ElementWrapper::new(document.get_element_by_id("battle").unwrap()),
            menu_scene: ElementWrapper::new(document.get_element_by_id("menu").unwrap()),
            document,
        }
    }
}
