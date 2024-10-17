use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Input {
    Enter,
    Cancel,
    Context,
    ArrowRight,
    ArrowLeft,
    ArrowUp,
    ArrowDown,
    None,
}

impl Input {
    pub fn from(key: String) -> Input {
        match key.as_str() {
            "a" => Input::Enter,
            "z" => Input::Cancel,
            "ArrowRight" => Input::ArrowRight,
            "ArrowLeft" => Input::ArrowLeft,
            "ArrowUp" => Input::ArrowUp,
            "ArrowDown" => Input::ArrowDown,
            _ => Input::None,
        }
    }
}
