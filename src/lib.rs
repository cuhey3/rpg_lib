pub mod engine;
mod features;
mod rpg;
mod svg;
mod utils;

use crate::engine::Engine;
use features::animation::Animation;
use svg::Position;
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

#[wasm_bindgen]
pub fn create_rpg_engine() -> Engine {
    rpg::mount()
}
