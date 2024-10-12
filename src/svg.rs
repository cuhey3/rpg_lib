use crate::svg::element_wrapper::ElementWrapper;
use web_sys::{Document, Element};

pub mod animation;
pub mod element_wrapper;

pub enum CursorType {
    Default,
    Side,
}
pub struct Cursor {
    element: Element,
    pub choose_index: usize,
    choice_length: usize,
    step_length: f64,
    default_x: f64,
    default_y: f64,
    cursor_type: CursorType,
}

impl Cursor {
    pub fn new(
        document: &Document,
        cursor_id: &str,
        choice_length: usize,
        step_length: f64,
    ) -> Cursor {
        let element = document.get_element_by_id(cursor_id).unwrap();
        let default_x = element.get_attribute("x").unwrap().parse().unwrap();
        let default_y = element.get_attribute("y").unwrap().parse().unwrap();
        Cursor {
            element,
            choose_index: 0,
            choice_length,
            step_length,
            default_x,
            default_y,
            cursor_type: CursorType::Default,
        }
    }
    pub fn set_cursor_type(&mut self, cursor_type: CursorType) {
        self.reset();
        self.cursor_type = cursor_type;
    }
    pub fn update_choice_length(&mut self, choice_length: usize) {
        self.choice_length = choice_length;
        self.choose_index = self.choose_index.min(self.choice_length - 1);
    }
    pub fn reset(&mut self) {
        self.choose_index = 0;
        match self.cursor_type {
            CursorType::Default => {
                self.element
                    .set_attribute("y", &*self.default_y.to_string())
                    .unwrap();
            }
            CursorType::Side => {
                self.element
                    .set_attribute("x", &*self.default_x.to_string())
                    .unwrap();
            }
        }
    }
    pub fn consume(&mut self, key: String) {
        let new_index = match self.cursor_type {
            CursorType::Default => match key.as_str() {
                "ArrowUp" => (self.choose_index + self.choice_length - 1) % self.choice_length,
                "ArrowDown" => (self.choose_index + 1) % self.choice_length,
                _ => self.choose_index,
            },
            CursorType::Side => match key.as_str() {
                "ArrowLeft" => (self.choose_index + self.choice_length - 1) % self.choice_length,
                "ArrowRight" => (self.choose_index + 1) % self.choice_length,
                _ => self.choose_index,
            },
        };
        self.choose_index = new_index;
        match self.cursor_type {
            CursorType::Default => {
                let new_y: f64 = self.default_y + new_index as f64 * self.step_length;
                self.element
                    .set_attribute("y", new_y.to_string().as_str())
                    .unwrap();
            }
            CursorType::Side => {
                let new_x: f64 = self.default_x + new_index as f64 * self.step_length;
                self.element
                    .set_attribute("x", new_x.to_string().as_str())
                    .unwrap();
            }
        }
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
