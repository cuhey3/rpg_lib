mod animation;
mod scene;
mod utils;

use crate::animation::Animation;
use crate::scene::battle::create_battle_scene;
use crate::scene::field::create_field_scene;
use crate::scene::menu::create_menu_scene;
use crate::scene::scene_type::SceneType::Field;
use crate::scene::title::create_title_scene;
use crate::ItemType::{Consumable, Weapon};
use rand::Rng;
use scene::Scene;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::__rt::console_log;
use wasm_bindgen_test::console_log;
use web_sys::WebSocket;
use web_sys::{Document, Element, MessageEvent};
struct SharedStatus {
    scene_index: usize,
    requested_scene_index: usize,
    map_index: usize,
    requested_map_index: usize,
    interrupt_animations: Vec<Vec<Animation>>,
    has_message: bool,
    elements: SharedElements,
    treasure_box_opened: Vec<Vec<usize>>,
    save_data: SaveData,
    web_socket_wrapper: WebSocketWrapper,
    online_users: Vec<PositionMessage>,
}

impl SharedStatus {
    fn update_save_data(&mut self, characters: &Vec<Character>) {
        self.save_data
            .update(characters, &self.treasure_box_opened, self.map_index);
    }
    fn load_save_data(&mut self, characters: &mut Vec<Character>) {
        self.save_data.load(characters, true);
        self.treasure_box_opened = self.save_data.treasure_box_usize.to_vec();
        self.map_index = *self.save_data.map_usize.get(0).unwrap();
        self.requested_map_index = *self.save_data.map_usize.get(0).unwrap();
    }
    fn new_game(&mut self, characters: &mut Vec<Character>) {
        let mut new_save_data = SaveData::empty();
        new_save_data.load(characters, false);
        self.treasure_box_opened = new_save_data.treasure_box_usize.to_vec();
        console_log!(
            "new_game map1 {} {}",
            self.map_index,
            new_save_data.map_usize.get(0).unwrap()
        );
        self.map_index = *new_save_data.map_usize.get(0).unwrap();
        self.requested_map_index = *new_save_data.map_usize.get(0).unwrap();
        console_log!(
            "new_game map2 {} {}",
            self.map_index,
            new_save_data.map_usize.get(0).unwrap()
        );
    }
}
struct SharedElements {
    message: ElementWrapper,
    document: Document,
    title_scene: ElementWrapper,
    field_scene: ElementWrapper,
    battle_scene: ElementWrapper,
    menu_scene: ElementWrapper,
}

impl SharedElements {
    fn new() -> SharedElements {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        SharedElements {
            message: ElementWrapper::new(document.get_element_by_id("message").unwrap()),
            title_scene: ElementWrapper::new(document.get_element_by_id("title").unwrap()),
            field_scene: ElementWrapper::new(document.get_element_by_id("field").unwrap()),
            battle_scene: ElementWrapper::new(document.get_element_by_id("battle").unwrap()),
            menu_scene: ElementWrapper::new(document.get_element_by_id("menu").unwrap()),
            document,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    character_u32: Vec<u32>,
    treasure_box_usize: Vec<Vec<usize>>,
    map_usize: Vec<usize>,
    map_i32: Vec<i32>,
    inventory_string: Vec<String>,
    check_token: u32,
}

impl SaveData {
    fn load(&mut self, characters: &mut Vec<Character>, try_get_storage: bool) {
        if try_get_storage {
            let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
            let raw_save = storage.get_item("save").unwrap();
            if raw_save.is_some() {
                let raw_save = raw_save.unwrap();
                let local_save_data: SaveData = serde_json::from_str(raw_save.as_str()).unwrap();
                self.character_u32 = local_save_data.character_u32.to_vec();
                self.treasure_box_usize = local_save_data.treasure_box_usize.to_vec();
                self.map_usize = local_save_data.map_usize.to_vec();
                self.map_i32 = local_save_data.map_i32.to_vec();
                self.inventory_string = local_save_data.inventory_string.to_vec();
                self.check_token = local_save_data.check_token.to_owned();
            }
        }
        characters[0].current_hp = *self.character_u32.get(0).unwrap();
        characters[0].max_hp = *self.character_u32.get(1).unwrap();
        characters[0].position.x = *self.map_i32.get(0).unwrap();
        characters[0].position.y = *self.map_i32.get(1).unwrap();
        characters[0].inventory = self
            .inventory_string
            .iter()
            .map(|s| Item::new(s.as_str()))
            .collect();
    }
    fn update(
        &mut self,
        characters: &Vec<Character>,
        treasure_box_opened: &Vec<Vec<usize>>,
        map_index: usize,
    ) {
        self.character_u32 = vec![characters[0].current_hp, characters[0].max_hp];
        self.treasure_box_usize = treasure_box_opened.to_vec();
        self.map_usize = vec![map_index];
        self.map_i32 = vec![characters[0].position.x, characters[0].position.y];
        self.inventory_string = characters[0]
            .inventory
            .iter()
            .map(|item| item.name.clone())
            .collect::<Vec<String>>();
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let json = serde_json::to_string(self).unwrap();
        storage.set_item("save", json.as_str()).unwrap();
    }
    fn new(
        character_u32: Vec<u32>,
        treasure_box_usize: Vec<Vec<usize>>,
        map_usize: Vec<usize>,
        map_i32: Vec<i32>,
        inventory_string: Vec<String>,
    ) -> SaveData {
        SaveData {
            character_u32,
            treasure_box_usize,
            map_usize,
            map_i32,
            inventory_string,
            check_token: 0,
        }
    }
    fn empty() -> SaveData {
        SaveData {
            character_u32: vec![25, 80],
            treasure_box_usize: vec![vec![]],
            map_usize: vec![0],
            map_i32: vec![360, 280],
            inventory_string: vec![],
            check_token: 0,
        }
    }
}

struct Character {
    current_hp: u32,
    max_hp: u32,
    position: Position,
    inventory: Vec<Item>,
}

struct Item {
    name: String,
    item_type: ItemType,
    consume_func: fn(&Item, &mut SharedStatus, &mut Vec<Character>),
    description: String,
}

impl Item {
    fn new(name: &str) -> Item {
        let item_type: ItemType = match name {
            "薬草" => Consumable,
            "棍棒" => Weapon,
            _ => panic!(),
        };
        let description = match name {
            "薬草" => "HPを30回復",
            "棍棒" => "粗悪な武器",
            _ => "",
        }
        .to_string();
        fn consume_func(item: &Item, _: &mut SharedStatus, characters: &mut Vec<Character>) {
            match item.item_type {
                Consumable => match item.name.as_str() {
                    "薬草" => {
                        characters[0].current_hp =
                            characters[0].max_hp.min(characters[0].current_hp + 30);
                    }
                    _ => {}
                },
                _ => return,
            }
        }
        Item {
            name: name.to_string(),
            item_type,
            consume_func,
            description,
        }
    }
}

enum ItemType {
    Weapon,
    Consumable,
}

#[wasm_bindgen]
pub struct Engine {
    characters: Vec<Character>,
    scenes: Vec<Scene>,
    shared_status: SharedStatus,
    web_socket_wrapper: WebSocketWrapper,
    instance_id: String,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Engine {
        let mut rng = rand::thread_rng();
        let random_number = rng.random::<u16>();
        let user_name = random_number.to_string();
        let web_socket_wrapper = WebSocketWrapper::new(user_name.to_owned());
        let mut shared_status = SharedStatus {
            has_message: false,
            scene_index: 0,
            requested_scene_index: 0,
            map_index: 0,
            requested_map_index: 0,
            interrupt_animations: vec![],
            elements: SharedElements::new(),
            treasure_box_opened: vec![],
            save_data: SaveData::empty(),
            web_socket_wrapper: web_socket_wrapper.clone(),
            online_users: vec![],
        };
        Engine {
            characters: vec![Character {
                current_hp: 25,
                max_hp: 80,
                position: Position { x: -1, y: -1 },
                inventory: vec![],
            }],
            scenes: vec![
                create_title_scene(&mut shared_status),
                create_field_scene(&mut shared_status),
                create_battle_scene(&mut shared_status),
                create_menu_scene(&mut shared_status),
            ],
            shared_status,
            web_socket_wrapper,
            instance_id: user_name.to_owned(),
        }
    }

    pub fn init(&mut self) {
        let init_func = self.scenes[0].init_func;
        init_func(
            &mut self.scenes[0],
            &mut self.shared_status,
            &mut self.characters,
        );
    }

    pub fn set_web_socket_instance(&mut self, web_socket: WebSocket) {
        self.web_socket_wrapper.update_web_socket(web_socket);
        self.shared_status.web_socket_wrapper = self.web_socket_wrapper.clone();
    }
    pub fn keydown(&mut self, key: String) {
        if self.shared_status.has_message {
            self.shared_status.elements.message.hide();
            self.shared_status.has_message = false;
            return;
        }
        if self.has_animation_blocking_scene_update() {
            console_log!("keydown interrupt {:?}", key);
            return;
        }
        let scene_index = self.shared_status.scene_index;
        let consume_func = self.scenes[scene_index].consume_func;
        console_log!("consume start scene: {:?}", scene_index);
        consume_func(
            &mut self.scenes[scene_index],
            &mut self.shared_status,
            &mut self.characters,
            key,
        );
        if !self.has_animation_blocking_scene_update() {
            if self.shared_status.scene_index != self.shared_status.requested_scene_index {
                self.shared_status.scene_index = self.shared_status.requested_scene_index;
                self.on_scene_update()
            }
            if self.shared_status.map_index != self.shared_status.requested_map_index {
                self.shared_status.map_index = self.shared_status.requested_map_index;
                self.on_map_update();
            }
        }
    }
    pub fn animate(&mut self, step: f64) {
        for animation in self.shared_status.interrupt_animations.iter_mut() {
            animation.get_mut(0).unwrap().set_step(step);
        }

        let mut to_delete_indexes = vec![];
        for (index, animation) in self
            .shared_status
            .interrupt_animations
            .iter_mut()
            .enumerate()
        {
            let func = animation.get(0).unwrap().animation_func;
            let result = func(
                animation.get_mut(0).unwrap(),
                self.shared_status.has_message,
                step,
            );
            if result {
                to_delete_indexes.push(index)
            }
        }

        to_delete_indexes.reverse();
        for index in to_delete_indexes.iter() {
            let at_animations = self
                .shared_status
                .interrupt_animations
                .get_mut(*index)
                .unwrap();
            at_animations.remove(0);
            if at_animations.is_empty() {
                self.shared_status.interrupt_animations.remove(*index);
            }
        }

        if self
            .shared_status
            .interrupt_animations
            .iter()
            .filter(|animation| animation.get(0).unwrap().block_scene_update)
            .collect::<Vec<&Vec<Animation>>>()
            .len()
            == 0
        {
            if self.shared_status.scene_index != self.shared_status.requested_scene_index {
                self.shared_status.scene_index = self.shared_status.requested_scene_index;
                self.on_scene_update()
            }
            if self.shared_status.map_index != self.shared_status.requested_map_index {
                self.shared_status.map_index = self.shared_status.requested_map_index;
                self.on_map_update();
            }
        }
    }
    fn on_scene_update(&mut self) {
        console_log!("scene_updated {:?}", self.shared_status.scene_index);
        let scene_index = self.shared_status.scene_index;
        if scene_index != 0 && !self.web_socket_wrapper.is_joined {
            self.web_socket_wrapper.join();
        }
        if scene_index == 0 && self.web_socket_wrapper.is_joined {
            self.web_socket_wrapper.left();
        }
        if scene_index != 3 {
            for (index, scene) in self.scenes.iter_mut().enumerate() {
                if index != scene_index {
                    scene.hide();
                }
            }
        }
        let init_func = self.scenes[scene_index].init_func;

        init_func(
            &mut self.scenes[scene_index],
            &mut self.shared_status,
            &mut self.characters,
        );
    }

    fn on_map_update(&mut self) {
        let scene = &mut self.scenes[self.shared_status.scene_index];
        match scene.scene_type {
            Field(ref mut field_status) => {
                field_status.update_map(&mut self.shared_status, &mut self.characters);
            }
            _ => {}
        }
    }

    fn has_animation_blocking_scene_update(&self) -> bool {
        self.shared_status
            .interrupt_animations
            .iter()
            .find(|animation| animation.get(0).unwrap().block_scene_update)
            .is_some()
    }
    pub fn receive_channel_message(&mut self, message: String) {
        if let Ok(mut channel_message) = serde_json::from_str::<ChannelMessage>(&message) {
            if channel_message.user_name == self.instance_id {
                return
            }
            for scene in self.scenes.iter_mut() {
                if let Scene {scene_type: Field(field_status), .. } = scene {
                    let mut message = channel_message.message.to_owned();
                    // TODO
                    // ネストしたJSONの扱い…
                    while let Ok(message_string) = serde_json::from_str::<String>(&message) {
                        message = message_string
                    }
                    channel_message.message = message;
                    field_status.consume_channel_message(&channel_message, &mut self.shared_status);
                }
            }
        };
    }
}

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

struct World {
    areas: Vec<Area>,
    start_position: Position,
    start_field_index: usize,
}

impl World {
    fn init() -> World {
        let field = Area {
            min_position: Position::new(0, 0),
            max_position: Position::new(100, 100),
            cell_events: vec![],
            walls: vec![],
        };
        World {
            areas: vec![field],
            start_field_index: 0,
            start_position: Position::new(50, 50),
        }
    }
    fn new_character() -> Character {
        Character {
            current_hp: 25,
            max_hp: 80,
            position: Position::new(-1, -1),
            inventory: vec![],
        }
    }
}

#[derive(Clone, Copy)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Position {
        Position { x, y }
    }
    fn new_vec(args: Vec<[i32; 2]>) -> Vec<Position> {
        args.iter()
            .map(|arg| Position::new(arg[0], arg[1]))
            .collect()
    }
    fn new_area(areas: Vec<[i32; 4]>) -> Vec<Position> {
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
    fn from_element(element: &Element) -> Position {
        let x: i32 = element.get_attribute("x").unwrap().parse().unwrap();
        let y: i32 = element.get_attribute("y").unwrap().parse().unwrap();
        Position { x, y }
    }
}

struct ElementWrapper {
    element: Element,
}

impl ElementWrapper {
    fn new(element: Element) -> ElementWrapper {
        ElementWrapper { element }
    }

    fn show(&self) {
        self.element.set_attribute("display", "block").unwrap();
    }
    fn hide(&self) {
        self.element.set_attribute("display", "none").unwrap();
    }
}

struct Cursor {
    element: Element,
    choose_index: usize,
    choice_length: usize,
    step_height: f64,
    default_y: f64,
}

impl Cursor {
    fn new(document: &Document, cursor_id: &str, choice_length: usize, step_height: f64) -> Cursor {
        let element = document.get_element_by_id(cursor_id).unwrap();
        let default_y = element.get_attribute("y").unwrap().parse().unwrap();
        Cursor {
            element,
            choose_index: 0,
            choice_length,
            step_height,
            default_y,
        }
    }
    fn update_choice_length(&mut self, choice_length: usize) {
        self.choice_length = choice_length;
        self.choose_index = self.choose_index.min(self.choice_length - 1);
    }
    fn reset(&mut self) {
        self.choose_index = 0;
        self.element
            .set_attribute("y", &*self.default_y.to_string())
            .unwrap();
    }
    fn consume(&mut self, key: String) {
        let new_index = match key.as_str() {
            "ArrowUp" => (self.choose_index + self.choice_length - 1) % self.choice_length,
            "ArrowDown" => (self.choose_index + 1) % self.choice_length,
            _ => panic!(),
        };
        self.choose_index = new_index;
        let new_y: f64 = self.default_y + new_index as f64 * self.step_height;
        self.element
            .set_attribute("y", new_y.to_string().as_str())
            .unwrap();
    }
}
#[derive(Serialize, Debug)]
struct ChannelUser {
    user_name: String,
    channel_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChannelMessage {
    user_name: String,
    message_type: MessageType,
    message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum MessageType {
    Message,
    Join,
    Left,
}
#[derive(Clone)]
#[wasm_bindgen]
struct WebSocketWrapper {
    ws: WebSocket,
    is_opened: bool,
    is_closed: bool,
    is_joined: bool,
    messages: Vec<ChannelMessage>,
    user_name: String,
}

impl WebSocketWrapper {
    pub fn new(user_name: String) -> WebSocketWrapper {
        let ws =
            WebSocket::new("https://rust-server-956911707039.asia-northeast1.run.app/ws").unwrap();
        let websocket_wrapper = WebSocketWrapper {
            ws: ws.clone(),
            is_opened: false,
            is_closed: false,
            is_joined: false,
            messages: vec![],
            user_name: user_name.to_owned(),
        };
        let mut clone = websocket_wrapper.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            let channel_user = ChannelUser {
                user_name: clone.user_name.to_owned(),
                channel_name: "rpg".to_string(),
            };
            clone
                .ws
                .send_with_str(&serde_json::to_string(&channel_user).unwrap())
                .unwrap();
            clone.is_opened = true;
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let mut clone = websocket_wrapper.clone();
        let onclose_callback = Closure::<dyn FnMut()>::new(move || {
            clone.is_closed = true;
        });
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        // let mut clone = websocket_wrapper.clone();
        // let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        //     if let Ok(raw_text) = e.data().dyn_into::<js_sys::JsString>() {
        //         let raw_text = raw_text.as_string().unwrap();
        //         let received_message: ChannelMessage = serde_json::from_str(&raw_text).unwrap();
        //         if let Ok(nested_message) =
        //             serde_json::from_str::<ChannelMessage>(&*received_message.message)
        //         {
        //             if nested_message.user_name != clone.user_name {
        //                 clone.messages.push(nested_message);
        //             }
        //         };
        //     }
        // });
        // ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        // onmessage_callback.forget();
        websocket_wrapper
    }

    pub fn is_ready(&self) -> bool {
        self.is_opened && !self.is_closed
    }
    pub fn join(&mut self) {
        let join_message = ChannelMessage {
            user_name: self.user_name.to_owned(),
            message_type: MessageType::Join,
            message: "join".to_string(),
        };
        self.ws
            .send_with_str(&serde_json::to_string(&join_message).unwrap())
            .unwrap();
        self.is_joined = true;
    }
    pub fn left(&mut self) {
        let left_message = ChannelMessage {
            user_name: self.user_name.to_owned(),
            message_type: MessageType::Left,
            message: "left".to_string(),
        };
        self.ws
            .send_with_str(&serde_json::to_string(&left_message).unwrap())
            .unwrap();
        self.is_joined = false;
    }
    pub fn send_message(&self, message: String) {
        self.ws
            .send_with_str(&serde_json::to_string(&message).unwrap())
            .unwrap();
    }
    // Need to wait for onopen event elsewhere
    pub fn update_web_socket(&mut self, web_socket: WebSocket) {
        self.ws = web_socket;
        let channel_user = ChannelUser {
            user_name: self.user_name.to_owned(),
            channel_name: "rpg".to_string(),
        };
        self
            .ws
            .send_with_str(&serde_json::to_string(&channel_user).unwrap()).unwrap()
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct PositionMessage {
    user_name: String,
    direction: String,
    position_x: i32,
    position_y: i32,
    map_index: usize,
}