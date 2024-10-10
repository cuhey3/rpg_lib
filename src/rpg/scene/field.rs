use crate::engine::SharedState;
use crate::rpg::scene::field::EventType::*;
use crate::rpg::scene::Scene;
use crate::rpg::scene::SceneType::Field;
use crate::rpg::Character;
use crate::ws::{ChannelMessage, MessageType, PositionMessage};
use crate::{Animation, Item, Position};
use wasm_bindgen_test::console_log;
use web_sys::{Document, Element};

pub struct FieldState {
    character_direction_element: Element,
    wrapper_element: Element,
    wrapper_translate_x: i32,
    wrapper_translate_y: i32,
    maps: Vec<Map>,
}

impl FieldState {
    pub fn move_to(
        &mut self,
        shared_state: &mut SharedState,
        characters: &mut Vec<Character>,
        key: String,
    ) {
        let map = &mut self.maps[shared_state.map_index];
        // let start_x: i32 = characters[0].position.x;
        // let start_y: i32 = characters[0].position.y;
        let mut x: i32 = characters[0].position.x.to_owned();
        let mut y: i32 = characters[0].position.y.to_owned();
        let original_translate_x = self.wrapper_translate_x.to_owned();
        let original_translate_y = self.wrapper_translate_y.to_owned();
        match key.as_str() {
            "ArrowUp" => {
                y -= 40;
                self.wrapper_translate_y += 40;
            }
            "ArrowDown" => {
                y += 40;
                self.wrapper_translate_y -= 40;
            }
            "ArrowRight" => {
                x += 40;
                self.wrapper_translate_x -= 40;
            }

            "ArrowLeft" => {
                x -= 40;
                self.wrapper_translate_x += 40;
            }
            _ => panic!(),
        }
        let found_event = map
            .event_positions
            .iter()
            .find(|(position, _)| position.x == x && position.y == y);
        if found_event.is_none() {
            match key.as_str() {
                "ArrowUp" | "ArrowDown" | "ArrowRight" | "ArrowLeft" => {
                    characters[0].position = Position::new(x, y);
                    self.update_character_position(x, y);
                    // self.character_position = Position::new(x, y);
                    // shared_state.interrupt_animations.push(vec![Animation::create_move(start_x, start_y, x, y)]);
                }
                _ => panic!(),
            }
            return;
        }
        let found_event = found_event.unwrap();
        match found_event.1 {
            Enemy => {
                shared_state.requested_scene_index += 1;
                shared_state
                    .interrupt_animations
                    .push(vec![Animation::create_fade_out_in()]);
                self.reset_translate(original_translate_x, original_translate_y);
                return;
            }
            TreasureBox => {
                let treasure_events = map
                    .event_positions
                    .iter()
                    .filter(|(_, event_type)| match event_type {
                        TreasureBox => true,
                        _ => false,
                    })
                    .collect::<Vec<&(Position, EventType)>>();
                let found_treasure_box = treasure_events
                    .iter()
                    .enumerate()
                    .find(|(_, tuple)| tuple.0.x == x && tuple.0.y == y);
                let treasure_index = found_treasure_box.unwrap().0;
                let opened = shared_state.treasure_box_opened[map.map_index]
                    .iter()
                    .find(|index| **index == treasure_index)
                    .is_some();
                if opened {
                    self.reset_translate(original_translate_x, original_translate_y);
                    return;
                }
                shared_state.treasure_box_opened[map.map_index].push(treasure_index);
                map.treasure_elements[treasure_index]
                    .set_attribute("fill", "gray")
                    .unwrap();
                shared_state.has_message = true;
                let item = map.treasure_items.get(treasure_index).unwrap();
                characters[0].inventory.push(Item::new(&item.name));
                shared_state
                    .interrupt_animations
                    .push(vec![Animation::create_message(format!(
                        "{}を手に入れた",
                        item.name
                    ))]);
                self.reset_translate(original_translate_x, original_translate_y);
                return;
            }
            Obstacle(..) => {
                self.reset_translate(original_translate_x, original_translate_y);
                return;
            }
            MapConnection(map_connection_detail) => {
                self.update_character_position(x, y);
                characters[0].position = Position::new(
                    map_connection_detail.to_position.x,
                    map_connection_detail.to_position.y,
                );
                self.reset_translate(original_translate_x, original_translate_y);
                shared_state.requested_map_index =
                    (shared_state.map_index as i32 + map_connection_detail.index_addition) as usize;
                shared_state
                    .interrupt_animations
                    .push(vec![Animation::create_fade_out_in()]);
                return;
            }
        }
    }

    pub fn reset_translate(&mut self, original_x: i32, original_y: i32) {
        self.wrapper_translate_x = original_x;
        self.wrapper_translate_y = original_y;
    }

    pub fn update_character_position(&mut self, x: i32, y: i32) {
        self.wrapper_translate_x = 360 - x;
        self.wrapper_translate_y = 280 - y;
        self.wrapper_element
            .set_attribute(
                "transform",
                format!(
                    "translate({}, {})",
                    self.wrapper_translate_x, self.wrapper_translate_y
                )
                .as_str(),
            )
            .unwrap();
    }
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut SharedState, &mut Vec<Character>) {
        fn init_func(
            scene: &mut Scene,
            shared_state: &mut SharedState,
            characters: &mut Vec<Character>,
        ) {
            shared_state.elements.field_scene.show();
            match &mut scene.scene_type {
                Field(field_state) => {
                    field_state.maps[shared_state.map_index].draw(shared_state);
                    field_state.update_character_position(
                        characters[0].position.x,
                        characters[0].position.y,
                    );
                }
                _ => {}
            }

            if characters[0].position.x == -1 && characters[0].position.y == -1 {
                characters[0].position = Position::new(360, 280);
            }

            console_log!("init end");
        }
        init_func
    }
    pub fn create_consume_func(
        &self,
    ) -> fn(&mut Scene, &mut SharedState, &mut Vec<Character>, String) {
        fn consume_func(
            scene: &mut Scene,
            shared_state: &mut SharedState,
            characters: &mut Vec<Character>,
            key: String,
        ) {
            match &mut scene.scene_type {
                Field(field_state) => {
                    let direction_string = match key.as_str() {
                        "ArrowUp" => "↑",
                        "ArrowDown" => "↓",
                        "ArrowRight" => "→",
                        "ArrowLeft" => "←",
                        _ => "",
                    };
                    if direction_string != "" {
                        field_state
                            .character_direction_element
                            .set_inner_html(direction_string);
                    }
                    match key.as_str() {
                        "ArrowUp" | "ArrowDown" | "ArrowRight" | "ArrowLeft" => {
                            field_state.move_to(shared_state, characters, key.to_owned());
                            let message = PositionMessage {
                                user_name: shared_state.user_name.to_owned(),
                                direction: key.to_owned(),
                                position_x: characters[0].position.x,
                                position_y: characters[0].position.y,
                                map_index: shared_state.requested_map_index,
                            };
                            shared_state
                                .to_send_channel_messages.push(serde_json::to_string(&message).unwrap());
                        }
                        "Escape" => {
                            shared_state.requested_scene_index += 2;
                        }
                        _ => (),
                    }
                }
                _ => panic!(),
            }
        }
        consume_func
    }
    pub fn update_map(&mut self, shared_state: &mut SharedState, characters: &mut Vec<Character>) {
        let map = &mut self.maps[shared_state.map_index];
        map.init_treasure_box_opened(shared_state);
        map.draw(shared_state);
        self.update_character_position(characters[0].position.x, characters[0].position.y);
    }

    pub fn consume_channel_message(
        &mut self,
        message: &ChannelMessage,
        shared_state: &mut SharedState,
    ) {
        let found = shared_state
            .online_users
            .iter_mut()
            .enumerate()
            .find(|(_, user)| user.user_name == message.user_name);
        match message.message_type {
            MessageType::Left => {
                if found.is_some() {
                    let remove_index = found.unwrap().0;
                    shared_state.online_users.remove(remove_index);
                }
            }
            MessageType::Message => {
                if let Ok(online_user) = serde_json::from_str::<PositionMessage>(&message.message) {
                    if found.is_some() {
                        let found = found.unwrap().1;
                        found.map_index = online_user.map_index;
                        found.direction = online_user.direction;
                        found.position_x = online_user.position_x;
                        found.position_y = online_user.position_y;
                    } else {
                        shared_state.online_users.push(online_user);
                    }
                } else if let Ok(message) = serde_json::from_str::<ChannelMessage>(&message.message)
                {
                    match message.message_type {
                        MessageType::Left => {
                            if found.is_some() {
                                let remove_index = found.unwrap().0;
                                shared_state.online_users.remove(remove_index);
                            }
                        }
                        _ => {}
                    }
                };
            }
            _ => {}
        }
        self.maps[shared_state.map_index].draw(shared_state);
    }
}

#[derive(Copy, Clone)]
struct MapConnectionDetail {
    index_addition: i32,
    from_position: Position,
    to_position: Position,
}

#[derive(Copy, Clone)]
enum ObstacleType {
    Rock,
    Lake,
}

impl ObstacleType {
    fn get_color(&self) -> String {
        match self {
            ObstacleType::Rock => "gray".to_string(),
            ObstacleType::Lake => "aqua".to_string(),
        }
    }
}

impl MapConnectionDetail {
    fn inverse(&self) -> MapConnectionDetail {
        MapConnectionDetail {
            index_addition: self.index_addition * -1,
            from_position: self.to_position,
            to_position: self.from_position,
        }
    }
}
struct Map {
    map_index: usize,
    event_positions: Vec<(Position, EventType)>,
    treasure_elements: Vec<Element>,
    treasure_items: Vec<Item>,
    ground_start_position: Position,
    ground_width: i32,
    ground_height: i32,
    ground_color: String,
}

impl Map {
    fn extract_events(
        result: &mut Vec<(Position, EventType)>,
        event_type: EventType,
        positions: Vec<Position>,
    ) -> &mut Vec<(Position, EventType)> {
        for position in positions {
            result.push((position, event_type.to_owned()));
        }
        result
    }
    pub fn events_to_elements(
        &mut self,
        document: &Document,
        parent: &Element,
        treasure_box_opened: &Vec<usize>,
    ) {
        let mut treasure_elements = vec![];
        let mut treasure_index = 0_usize;
        for (position, event_type) in self.event_positions.iter() {
            let rect_color = match event_type {
                TreasureBox => {
                    let opened = treasure_box_opened
                        .iter()
                        .find(|contained_index| **contained_index == treasure_index);
                    treasure_index += 1;
                    if opened.is_some() {
                        "gray"
                    } else {
                        "orange"
                    }
                }
                Enemy => "red",
                Obstacle(obstacle_type) => &*obstacle_type.get_color(),
                MapConnection(..) => "black",
            };
            let inner_html = match event_type {
                TreasureBox => "宝",
                Enemy => "敵",
                Obstacle(..) => "",
                MapConnection(..) => "black",
            };
            let class_name = match event_type {
                TreasureBox => "treasure-box",
                Enemy => "enemy",
                Obstacle(..) => "obstacle",
                MapConnection(..) => "map-connection",
            };
            let rect = document
                .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "rect")
                .unwrap();
            rect.set_attribute("x", &*position.x.to_string()).unwrap();
            rect.set_attribute("y", &*position.y.to_string()).unwrap();
            rect.set_attribute("fill", rect_color).unwrap();
            rect.set_attribute("width", "40").unwrap();
            rect.set_attribute("height", "40").unwrap();
            rect.class_list().add_2("_object", class_name).unwrap();
            parent.append_child(&*rect).unwrap();
            match event_type {
                TreasureBox | Enemy => {
                    let text = document
                        .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "text")
                        .unwrap();
                    text.set_attribute("x", &*(position.x + 2).to_string())
                        .unwrap();
                    text.set_attribute("y", &*(position.y + 33).to_string())
                        .unwrap();
                    text.set_attribute("font-size", "35").unwrap();
                    text.set_attribute("fill", "black").unwrap();
                    text.class_list().add_2("direction", class_name).unwrap();
                    text.set_inner_html(inner_html);
                    parent.append_child(&*text).unwrap();
                }
                _ => {}
            }
            match event_type {
                TreasureBox => treasure_elements.push(rect),
                _ => {}
            }
        }

        self.treasure_elements = treasure_elements
    }
    fn init_1() -> Map {
        let treasure_items = vec![Item::new("薬草")];
        let map_connection = MapConnectionDetail {
            index_addition: 1,
            from_position: Position::new(120, -40),
            to_position: Position::new(-440, -360),
        };
        let map_connection2 = MapConnectionDetail {
            index_addition: 2,
            from_position: Position::new(600, 240),
            to_position: Position::new(-440, -360),
        };
        let event_positions = &mut vec![];
        Map::extract_events(
            event_positions,
            Enemy,
            Position::new_vec(vec![[320, -40], [80, 80], [440, 320]]),
        );
        Map::extract_events(
            event_positions,
            TreasureBox,
            Position::new_vec(vec![[320, 120]]),
        );
        Map::extract_events(
            event_positions,
            Obstacle(ObstacleType::Rock),
            Position::new_area(vec![[80, -80, 160, -40], [560, 200, 640, 240]]),
        );
        Map::extract_events(
            event_positions,
            Obstacle(ObstacleType::Lake),
            Position::new_vec(vec![
                [-40, 240],
                [0, 240],
                [40, 240],
                [80, 240],
                [-80, 280],
                [-40, 280],
                [0, 280],
                [40, 280],
                [80, 280],
                [120, 280],
                [-40, 320],
                [0, 320],
                [40, 320],
                [80, 320],
            ]),
        );
        Map::extract_events(
            event_positions,
            MapConnection(map_connection),
            vec![map_connection.from_position.to_owned()],
        );
        Map::extract_events(
            event_positions,
            MapConnection(map_connection2),
            vec![map_connection2.from_position.to_owned()],
        );
        let map = Map {
            map_index: 0,
            event_positions: event_positions.to_vec(),
            treasure_elements: vec![],
            treasure_items,
            ground_start_position: Position::new(-4000, -3000),
            ground_width: 8000,
            ground_height: 6000,
            ground_color: "#996633".to_string(),
        };
        map
    }

    fn init_2() -> Map {
        let treasure_items = vec![Item::new("棍棒")];
        let map_connection_detail = MapConnectionDetail {
            index_addition: 1,
            from_position: Position::new(120, -40),
            to_position: Position::new(-440, -360),
        }
        .inverse();
        let event_positions = &mut vec![];
        Map::extract_events(
            event_positions,
            TreasureBox,
            vec![Position::new(-480, -480)],
        );
        Map::extract_events(
            event_positions,
            Obstacle(ObstacleType::Rock),
            Position::new_area(vec![[-600, -600, -320, -360]]),
        );
        Map::extract_events(
            event_positions,
            MapConnection(map_connection_detail),
            Position::new_vec(vec![
                [-360, -320],
                [-400, -320],
                [-440, -320],
                [-480, -320],
                [-520, -320],
                [-560, -320],
            ]),
        );
        let map = Map {
            map_index: 1,
            event_positions: event_positions.to_vec(),
            treasure_elements: vec![],
            treasure_items,
            ground_start_position: Position::new(-600, -600),
            ground_width: 320,
            ground_height: 280,
            ground_color: "#663300".to_string(),
        };
        map
    }
    fn init_3() -> Map {
        let treasure_items = vec![];
        let map_connection_detail = MapConnectionDetail {
            index_addition: 2,
            from_position: Position::new(600, 240),
            to_position: Position::new(-440, -360),
        }
        .inverse();
        let event_positions = &mut vec![];
        Map::extract_events(
            event_positions,
            Enemy,
            Position::new_vec(vec![[-520, -1120]]),
        );
        Map::extract_events(
            event_positions,
            Obstacle(ObstacleType::Rock),
            Position::new_area(vec![[-600, -1200, -320, -360]]),
        );
        Map::extract_events(
            event_positions,
            MapConnection(map_connection_detail),
            Position::new_vec(vec![
                [-360, -320],
                [-400, -320],
                [-440, -320],
                [-480, -320],
                [-520, -320],
                [-560, -320],
            ]),
        );

        let map = Map {
            map_index: 2,
            event_positions: event_positions.to_vec(),
            treasure_elements: vec![],
            treasure_items,
            ground_start_position: Position::new(-600, -1200),
            ground_width: 320,
            ground_height: 880,
            ground_color: "#663300".to_string(),
        };
        map
    }
    fn init_treasure_box_opened(&mut self, shared_state: &mut SharedState) {
        let treasure_box_opened = &mut shared_state.treasure_box_opened;
        while treasure_box_opened.len() <= self.map_index {
            treasure_box_opened.push(vec![]);
        }
    }
    fn draw(&mut self, shared_state: &SharedState) {
        let ref document = shared_state.elements.document;
        let wrapper_element = document.query_selector("#field-wrapper").unwrap().unwrap();
        while {
            let child = wrapper_element.child_nodes().get(0);
            if child.is_none() {
                false
            } else {
                wrapper_element.remove_child(&child.unwrap()).unwrap();
                true
            }
        } {}
        let ground = document
            .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "rect")
            .unwrap();
        ground
            .set_attribute("x", &*self.ground_start_position.x.to_string())
            .unwrap();
        ground
            .set_attribute("y", &*self.ground_start_position.y.to_string())
            .unwrap();
        ground
            .set_attribute("fill", &*self.ground_color.to_string())
            .unwrap();
        ground
            .set_attribute("width", &*self.ground_width.to_string())
            .unwrap();
        ground
            .set_attribute("height", &*self.ground_height.to_string())
            .unwrap();
        wrapper_element.append_child(&*ground).unwrap();
        let treasure_box_opened = &shared_state.treasure_box_opened[self.map_index];
        self.events_to_elements(document, &wrapper_element, treasure_box_opened);
        self.draw_online_user(shared_state);
    }
    pub fn draw_online_user(&mut self, shared_state: &SharedState) {
        let map_index = self.map_index;
        for user in shared_state.online_users.iter() {
            if user.map_index != map_index {
                continue;
            }
            let ref document = shared_state.elements.document;
            let wrapper_element = document.query_selector("#field-wrapper").unwrap().unwrap();
            let rect = document
                .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "rect")
                .unwrap();
            rect.set_attribute("x", &*user.position_x.to_string())
                .unwrap();
            rect.set_attribute("y", &*user.position_y.to_string())
                .unwrap();
            rect.set_attribute("fill", "white").unwrap();
            rect.set_attribute("width", "40").unwrap();
            rect.set_attribute("height", "40").unwrap();
            rect.class_list().add_1("online-user").unwrap();
            wrapper_element.append_child(&*rect).unwrap();
            let text = document
                .create_element_ns(Option::from("http://www.w3.org/2000/svg"), "text")
                .unwrap();
            text.set_attribute("x", &*(user.position_x + 2).to_string())
                .unwrap();
            text.set_attribute("y", &*(user.position_y + 33).to_string())
                .unwrap();
            text.set_attribute("font-size", "35").unwrap();
            text.set_attribute("fill", "black").unwrap();
            text.class_list().add_2("direction", "online-user").unwrap();
            text.set_inner_html(match user.direction.as_str() {
                "ArrowRight" => "→",
                "ArrowLeft" => "←",
                "ArrowUp" => "↑",
                "ArrowDown" => "↓",
                _ => "",
            });
            wrapper_element.append_child(&*text).unwrap();
        }
    }
}
#[derive(Clone)]
enum EventType {
    Enemy,
    TreasureBox,
    Obstacle(ObstacleType),
    MapConnection(MapConnectionDetail),
}

pub fn create_field_scene(shared_state: &mut SharedState) -> Scene {
    let mut map = Map::init_1();
    map.init_treasure_box_opened(shared_state);
    map.draw(shared_state);
    let character_direction_element = shared_state
        .elements
        .document
        .query_selector(".character.direction")
        .unwrap()
        .unwrap();
    let field_state = FieldState {
        character_direction_element,
        wrapper_element: shared_state
            .elements
            .document
            .query_selector("#field-wrapper")
            .unwrap()
            .unwrap(),
        wrapper_translate_x: 0,
        wrapper_translate_y: 0,
        maps: vec![map, Map::init_2(), Map::init_3()],
    };
    let consume_func = field_state.create_consume_func();
    let init_func = field_state.create_init_func();
    let scene_type = Field(field_state);
    Scene {
        element_id: "field".to_string(),
        scene_type,
        consume_func,
        init_func,
    }
}
