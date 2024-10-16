use web_sys::Element;

pub struct ElementWrapper {
    pub element: Element,
}

impl ElementWrapper {
    pub fn new(element: Element) -> ElementWrapper {
        ElementWrapper { element }
    }

    pub fn show(&self) {
        self.element.set_attribute("display", "block").unwrap();
    }
    pub fn hide(&self) {
        self.element.set_attribute("display", "none").unwrap();
    }
}
