use crate::svg::animation::Animation;
use crate::svg::element_wrapper::ElementWrapper;
use crate::ws::{PositionMessage, WebSocketWrapper};
use crate::Position;
use serde::{Deserialize, Serialize};
use wasm_bindgen_test::console_log;
use web_sys::Document;

pub mod scene;

pub struct SharedState {
    pub scene_index: usize,
    pub requested_scene_index: usize,
    pub map_index: usize,
    pub requested_map_index: usize,
    pub interrupt_animations: Vec<Vec<Animation>>,
    pub has_message: bool,
    pub elements: SharedElements,
    pub treasure_box_opened: Vec<Vec<usize>>,
    pub save_data: SaveData,
    pub web_socket_wrapper: WebSocketWrapper,
    pub online_users: Vec<PositionMessage>,
}

impl SharedState {
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
pub struct SharedElements {
    pub message: ElementWrapper,
    document: Document,
    title_scene: ElementWrapper,
    field_scene: ElementWrapper,
    battle_scene: ElementWrapper,
    menu_scene: ElementWrapper,
}

impl SharedElements {
    pub fn new() -> SharedElements {
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

pub struct Item {
    pub name: String,
    item_type: ItemType,
    consume_func: fn(&Item, &mut SharedState, &mut Vec<Character>),
    description: String,
}

impl Item {
    pub fn new(name: &str) -> Item {
        let item_type: ItemType = match name {
            "薬草" => ItemType::Consumable,
            "棍棒" => ItemType::Weapon,
            _ => panic!(),
        };
        let description = match name {
            "薬草" => "HPを30回復",
            "棍棒" => "粗悪な武器",
            _ => "",
        }
        .to_string();
        fn consume_func(item: &Item, _: &mut SharedState, characters: &mut Vec<Character>) {
            match &item.item_type {
                ItemType::Consumable => match item.name.as_str() {
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

pub enum ItemType {
    Weapon,
    Consumable,
}

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    character_u32: Vec<u32>,
    pub treasure_box_usize: Vec<Vec<usize>>,
    pub map_usize: Vec<usize>,
    map_i32: Vec<i32>,
    inventory_string: Vec<String>,
    check_token: u32,
}

impl SaveData {
    pub fn load(&mut self, characters: &mut Vec<Character>, try_get_storage: bool) {
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
    pub fn update(
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
    pub fn empty() -> SaveData {
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

pub struct Character {
    pub current_hp: u32,
    pub max_hp: u32,
    pub position: Position,
    pub inventory: Vec<Item>,
}
