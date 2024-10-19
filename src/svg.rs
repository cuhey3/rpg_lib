use crate::svg::element_wrapper::ElementWrapper;
use web_sys::Document;

pub mod element_wrapper;
pub mod svg_renderer;

pub struct SharedElements {
    pub message: ElementWrapper,
    pub document: Document,
}

impl SharedElements {
    pub fn new() -> SharedElements {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        SharedElements {
            message: ElementWrapper::new(document.get_element_by_id("message").unwrap()),
            document,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position { x, y }
    }
    pub fn new_vec(args: Vec<[i32; 2]>) -> Vec<Position> {
        args.iter()
            .map(|arg| Position::new(arg[0], arg[1]))
            .collect()
    }
    pub fn new_area(areas: Vec<[i32; 4]>) -> Vec<Position> {
        let mut result = vec![];
        for area in areas.iter() {
            let [start_x, start_y, end_x, end_y] = *area;
            let step_x = (end_x - start_x) / 40;
            let step_y = (end_y - start_y) / 40;
            for y in 0..step_y + 1 {
                if y == 0 {
                    for x in 0..step_x + 1 {
                        result.push(Position::new(start_x + x * 40, start_y + y * 40))
                    }
                } else if y == end_y {
                } else {
                    result.push(Position::new(start_x, start_y + y * 40));
                    result.push(Position::new(end_x, start_y + y * 40));
                }
            }
        }
        result
    }
}
