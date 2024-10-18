pub mod engine;
mod features;
mod rpg;
mod svg;
mod utils;

use crate::engine::Engine;
use features::animation::Animation;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn create_rpg_engine() -> Engine {
    rpg::mount()
}
