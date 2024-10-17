use crate::rpg::mechanism::item::Item;
use crate::svg::Position;

pub struct Character {
    pub current_hp: u32,
    pub max_hp: u32,
    pub position: Position,
    pub inventory: Vec<Item>,
    pub event_flags: Vec<bool>,
}
