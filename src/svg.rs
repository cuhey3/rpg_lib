use crate::engine::Input;
use crate::svg::element_wrapper::ElementWrapper;
use wasm_bindgen_test::console_log;
use web_sys::{Document, Element};

pub mod animation;
pub mod element_wrapper;

pub enum CursorType {
    Default,
    Side,
    Box,
}
pub struct Cursor {
    pub element: Element,
    pub choose_index: usize,
    choice_length: usize,
    step_length: f64,
    default_x: f64,
    default_y: f64,
    box_x_length: usize,
    box_y_length: usize,
    pub cursor_type: CursorType,
}

impl Cursor {
    pub fn empty() -> Cursor {
        let element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "text")
            .unwrap();
        Cursor {
            element,
            choose_index: 0,
            choice_length: 0,
            step_length: 0.0,
            default_x: 0.0,
            default_y: 0.0,
            box_x_length: 0,
            box_y_length: 0,
            cursor_type: CursorType::Default,
        }
    }
    pub fn new_with_element(element: Element, step_length: f64) -> Cursor {
        Cursor {
            choose_index: 0,
            // 後から更新可能
            choice_length: 0,
            box_x_length: 0,
            box_y_length: 0,
            step_length,
            default_x: element.get_attribute("x").unwrap().parse().unwrap(),
            default_y: element.get_attribute("y").unwrap().parse().unwrap(),
            element,
            cursor_type: CursorType::Default,
        }
    }
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
            box_x_length: 0,
            box_y_length: 0,
            cursor_type: CursorType::Default,
        }
    }
    pub fn set_cursor_type(&mut self, cursor_type: CursorType) {
        self.reset();
        self.cursor_type = cursor_type;
    }
    pub fn index_to_box_x_box_y(&self, index: usize) -> (usize, usize) {
        (index % self.box_x_length, index / self.box_x_length)
    }
    pub fn update_choice_length(&mut self, choice_length: usize) {
        self.choice_length = choice_length;
        self.choose_index = self.choose_index.min(self.choice_length - 1);
    }
    pub fn set_box_length(&mut self, x_length: usize, y_length: usize) {
        self.box_x_length = x_length;
        self.box_y_length = y_length;
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
            CursorType::Box => {
                self.element
                    .set_attribute("x", &*self.default_x.to_string())
                    .unwrap();
                self.element
                    .set_attribute("y", &*self.default_y.to_string())
                    .unwrap();
            }
        }
    }
    pub fn consume(&mut self, input: Input) {
        let new_index = match self.cursor_type {
            CursorType::Default => match input {
                Input::ArrowUp => (self.choose_index + self.choice_length - 1) % self.choice_length,
                Input::ArrowDown => (self.choose_index + 1) % self.choice_length,
                _ => self.choose_index,
            },
            CursorType::Side => match input {
                Input::ArrowLeft => {
                    (self.choose_index + self.choice_length - 1) % self.choice_length
                }
                Input::ArrowRight => (self.choose_index + 1) % self.choice_length,
                _ => self.choose_index,
            },
            CursorType::Box => {
                let (mut x, mut y) = self.index_to_box_x_box_y(self.choose_index);
                console_log!("start x, y {}, {}", x, y);
                match input {
                    Input::ArrowUp => y = (y + self.box_y_length - 1) % self.box_y_length,
                    Input::ArrowDown => y = (y + 1) % self.box_y_length,
                    Input::ArrowLeft => x = (x + self.box_x_length - 1) % self.box_x_length,
                    Input::ArrowRight => {
                        x = (x + 1) % self.box_x_length;
                        if y * self.box_x_length + x > self.choice_length {
                            x = 0
                        }
                    }
                    _ => {}
                };
                let expect_index = y * self.box_x_length + x;
                expect_index.min(self.choice_length)
            }
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
            CursorType::Box => {
                let (x, y) = self.index_to_box_x_box_y(self.choose_index);
                let new_x: f64 = self.default_x + x as f64 * self.step_length;
                self.element
                    .set_attribute("x", new_x.to_string().as_str())
                    .unwrap();
                let new_y: f64 = self.default_y + y as f64 * self.step_length;
                self.element
                    .set_attribute("y", new_y.to_string().as_str())
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
