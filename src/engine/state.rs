use crate::engine::application_types::StateType;
use crate::features::animation::Animation;
use crate::svg::SharedElements;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Primitives {
    pub scene_index: usize,
    pub requested_scene_index: usize,
    pub map_index: usize,
    pub requested_map_index: usize,
}

pub struct References {
    pub has_block_message: bool,
    pub has_continuous_message: bool,
}

pub struct State {
    pub user_name: String,
    pub to_send_channel_messages: Vec<String>,
    pub state_type: StateType,
    pub elements: SharedElements,
    pub interrupt_animations: Vec<Vec<Animation>>,
    pub primitives: Primitives,
    pub references: Rc<RefCell<References>>,
}
