use crate::engine::{Choice, ChoiceSetting, ChoiceTree, Input};
use crate::rpg::ChoiceKind;
use crate::rpg::ChoiceKind::ChoseNth;
use crate::svg::element_wrapper::ElementWrapper;
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
        self.cursor_type = CursorType::Box;
    }
    pub fn reset(&mut self) {
        // TODO
        // カーソル位置を記憶する実装
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

pub struct SvgRenderer {
    choice_kind: ChoiceKind,
    target_part_name: String,
    wrapper_element: Option<Element>,
    item_element: Option<Element>,
    message_wrapper_element: Option<Element>,
    message_element: Option<Element>,
    pub cursor: Cursor,
    step_length: f64,
    item_labels: Vec<String>,
    item_x: f64,
    item_y: f64,
}

impl SvgRenderer {
    pub fn new(choice_kind: ChoiceKind, target_part_name: String, step_length: f64) -> SvgRenderer {
        let mut renderer = SvgRenderer {
            choice_kind,
            target_part_name,
            wrapper_element: None,
            item_element: None,
            message_wrapper_element: None,
            message_element: None,
            cursor: Cursor::empty(),
            step_length,
            item_labels: vec![],
            item_x: 0.0,
            item_y: 0.0,
        };
        renderer.load();
        renderer
    }
    pub fn load(&mut self) {
        self.load_wrapper_element();
        self.load_item_element();
        self.load_cursor();
        self.load_message_wrapper_element();
        self.load_message_element();
    }
    pub fn load_wrapper_element(&mut self) {
        self.wrapper_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_wrapper_id())
    }
    pub fn get_wrapper_id(&self) -> String {
        format!("render-{}-wrapper", self.target_part_name)
    }

    pub fn load_cursor(&mut self) {
        let cursor_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_cursor_id())
            .unwrap();
        self.cursor = Cursor::new_with_element(cursor_element, self.step_length);
    }

    pub fn get_cursor_id(&self) -> String {
        format!("render-{}-cursor", self.target_part_name)
    }

    pub fn load_item_element(&mut self) {
        self.item_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_item_id());
        if let Some(element) = &self.item_element {
            self.item_x = element.get_attribute("x").unwrap().parse().unwrap();
            self.item_y = element.get_attribute("y").unwrap().parse().unwrap();
        }
    }
    pub fn get_item_id(&self) -> String {
        format!("render-{}-item", self.target_part_name)
    }
    pub fn load_message_wrapper_element(&mut self) {
        self.message_wrapper_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_message_wrapper_id())
    }
    pub fn get_message_wrapper_id(&self) -> String {
        format!("render-{}-message-wrapper", self.target_part_name)
    }

    pub fn load_message_element(&mut self) {
        self.message_element = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id(&self.get_message_id())
    }
    pub fn get_message_id(&self) -> String {
        format!("render-{}-message", self.target_part_name)
    }

    pub fn get_rendered_id(&self) -> String {
        format!("render-{}-rendered", self.target_part_name)
    }

    pub fn render(&mut self, labels: Vec<String>, description: &str) {
        self.item_labels = labels;
        let document = web_sys::window().unwrap().document().unwrap();
        if let Some(to_remove) = document.get_element_by_id(self.get_rendered_id().as_str()) {
            to_remove.remove();
        }
        let group_element = document
            .create_element_ns(Some("http://www.w3.org/2000/svg"), "g")
            .unwrap();
        group_element
            .set_attribute("id", self.get_rendered_id().as_str())
            .unwrap();
        if let Some(wrapper_element) = &self.wrapper_element {
            wrapper_element.append_child(&*group_element).unwrap();
            wrapper_element.set_attribute("display", "block").unwrap();
        }

        for (index, label) in self.item_labels.iter().enumerate() {
            if let Some(item_element) = &self.item_element {
                let node = item_element.clone_node().unwrap();
                let empty_element = document
                    .create_element_ns(Some("http://www.w3.org/2000/svg"), "text")
                    .unwrap();
                node.append_child(&*empty_element).unwrap();
                let element = empty_element.parent_element().unwrap();
                element.set_inner_html(label);
                match self.cursor.cursor_type {
                    CursorType::Default => {
                        element
                            .set_attribute("x", &*self.item_x.to_string())
                            .unwrap();
                        element
                            .set_attribute(
                                "y",
                                &*(self.item_y + index as f64 * self.step_length).to_string(),
                            )
                            .unwrap();
                    }
                    CursorType::Box => {
                        let (x, y) = self.cursor.index_to_box_x_box_y(index);
                        element
                            .set_attribute(
                                "x",
                                &*(self.item_x + x as f64 * self.step_length).to_string(),
                            )
                            .unwrap();
                        element
                            .set_attribute(
                                "y",
                                &*(self.item_y + y as f64 * self.step_length).to_string(),
                            )
                            .unwrap();
                    }
                    _ => {}
                }
                element.set_attribute("display", "block").unwrap();
                group_element.append_child(&*element).unwrap();
            }
        }
        self.cursor.reset();
        self.cursor
            .element
            .set_attribute("display", "block")
            .unwrap();
        if let Some(element) = &self.message_wrapper_element {
            let display = if description.is_empty() {
                "none"
            } else {
                "block"
            };
            element.set_attribute("display", display).unwrap();
        }
        if let Some(element) = &self.message_element {
            if !description.is_empty() {
                element.set_inner_html(description);
            }
        }
    }
    pub fn hide(&self) {
        if let Some(element) = &self.wrapper_element {
            element.set_attribute("display", "none").unwrap();
        }
    }
}

pub struct RendererController {
    pub renderers: Vec<SvgRenderer>,
    pub choice_setting: ChoiceSetting,
    pub choice_tree: ChoiceTree,
    pub confirm_index: Option<usize>,
}

impl RendererController {
    pub fn now_choice_kind(&self) -> ChoiceKind {
        self.choice_tree.get_now()
    }

    pub fn undo_choice_tree(&mut self) {
        self.choice_tree.undo()
    }

    pub fn initial_render(&mut self) {
        self.choice_tree = self.choice_setting.get_menu_choice_tree();
        let labels = self.choice_tree.now_choice.get_branch_labels();
        self.renderers[0].cursor.update_choice_length(labels.len());
        let description = self
            .choice_tree
            .now_choice
            .branch_description
            .clone()
            .unwrap_or("".to_string());
        self.renderers[0].render(labels, description.as_str());
    }

    pub fn render_with(&mut self, labels: Vec<String>, description: &str) {
        let kind = self.choice_tree.get_now().clone();
        if let Some(renderer) = self.renderers.iter_mut().find(|r| r.choice_kind == kind) {
            renderer.cursor.reset();
            renderer.cursor.update_choice_length(labels.len());
            renderer.render(labels, description);
        }
    }
    pub fn delegate_input(&mut self, input: Input) {
        let kind = self.choice_tree.get_now().clone();
        if let Some(renderer) = self.renderers.iter_mut().find(|r| r.choice_kind == kind) {
            renderer.cursor.consume(input);
        }
    }

    pub fn delegate_enter(&mut self) {
        let kind = self.choice_tree.get_now().clone();
        if let Some(renderer) = self.renderers.iter_mut().find(|r| r.choice_kind == kind) {
            if let Some(branch) = &self.choice_tree.now_choice.branch {
                if let Some(
                    Choice {
                        own_token: ChoseNth(..),
                        ..
                    },
                    ..,
                ) = branch.get(0)
                {
                    self.choice_tree.choose(renderer.cursor.choose_index);
                }
            }
            self.choice_tree.choose(renderer.cursor.choose_index);
        }
    }

    pub fn delegate_close(&mut self) {
        let kind = self.choice_tree.get_now().clone();
        if let Some(renderer) = self.renderers.iter_mut().find(|r| r.choice_kind == kind) {
            renderer.hide();
            renderer.cursor.reset();
        }
        self.undo_choice_tree()
    }

    pub fn delegate_confirm(&mut self) {
        if self.confirm_index.is_none() {
            // confirm renderer not set
            return;
        }
        let description = self
            .choice_tree
            .now_choice
            .branch_description
            .clone()
            .unwrap();
        // ChoiceKind::Confirm に進む
        self.choice_tree.choose(0);

        let labels = self.choice_tree.now_choice.get_branch_labels();
        let confirm_index = self.confirm_index.unwrap();
        self.renderers[confirm_index].load();
        self.renderers[confirm_index]
            .cursor
            .update_choice_length(labels.len());
        self.renderers[confirm_index].render(labels, description.as_str());
    }
    pub fn close_all(&mut self) {
        for renderer in self.renderers.iter_mut() {
            renderer.hide();
            renderer.cursor.reset();
        }
    }

    pub fn get_chose_nth(&self) -> Option<usize> {
        for token in self.choice_tree.chose_kinds.iter() {
            if let ChoseNth(_, index) = token {
                return index.clone();
            }
        }
        None
    }
}

#[derive(Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
