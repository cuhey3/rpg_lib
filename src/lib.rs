mod application_types;
pub mod engine;
mod rpg;
mod svg;
mod utils;
mod ws;

use crate::engine::Engine;
use crate::rpg::Item;
use svg::animation::Animation;
use wasm_bindgen::prelude::wasm_bindgen;

struct Area {
    min_position: Position,
    max_position: Position,
    cell_events: Vec<CellEvent>,
    walls: Vec<Position>,
}

struct CellEvent {
    event_type: EventType,
    position: Position,
}

enum EventType {
    Transition(Area, Position),
    Shop,
}

#[derive(Clone, Copy)]
pub struct Position {
    x: i32,
    y: i32,
}

#[wasm_bindgen]
pub fn create_rpg_engine() -> Engine {
    rpg::mount()
}
