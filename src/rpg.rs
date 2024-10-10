use crate::application_types::{ApplicationType, StateType};
use crate::engine::{Engine, SharedElements, SharedState, State};
use crate::rpg::scene::battle::create_battle_scene;
use crate::rpg::scene::field::create_field_scene;
use crate::rpg::scene::menu::create_menu_scene;
use crate::rpg::scene::title::create_title_scene;
use crate::ws::{PositionMessage, WebSocketWrapper};
use crate::Position;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::svg::animation::Animation;

pub mod scene;

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

pub struct RPGSharedState {
    pub user_name: String,
    pub scene_index: usize,
    pub requested_scene_index: usize,
    pub map_index: usize,
    pub requested_map_index: usize,
    pub interrupt_animations: Vec<Vec<Animation>>,
    pub has_message: bool,
    pub elements: SharedElements,
    pub treasure_box_opened: Vec<Vec<usize>>,
    pub save_data: SaveData,
    pub online_users: Vec<PositionMessage>,
    pub to_send_channel_messages: Vec<String>
}

pub fn mount() -> Engine {
    let mut rng = rand::thread_rng();
    let random_number = rng.random::<u16>();
    let user_name = random_number.to_string();
    let web_socket_wrapper = WebSocketWrapper::new(user_name.to_owned());
    let mut shared_state = SharedState {
        user_name: user_name.to_owned(),
        has_message: false,
        scene_index: 0,
        requested_scene_index: 0,
        map_index: 0,
        requested_map_index: 0,
        interrupt_animations: vec![],
        elements: SharedElements::new(),
        treasure_box_opened: vec![],
        save_data: SaveData::empty(),
        online_users: vec![],
        to_send_channel_messages: vec![],
    };
    let mut rpg_shared_state = RPGSharedState {
        user_name: user_name.to_owned(),
        has_message: false,
        scene_index: 0,
        requested_scene_index: 0,
        map_index: 0,
        requested_map_index: 0,
        interrupt_animations: vec![],
        elements: SharedElements::new(),
        treasure_box_opened: vec![],
        save_data: SaveData::empty(),
        online_users: vec![],
        to_send_channel_messages: vec![],
    };
    let scenes = vec![
        create_title_scene(&mut shared_state),
        create_field_scene(&mut shared_state),
        create_battle_scene(&mut shared_state),
        create_menu_scene(&mut shared_state),
    ];
    let mut engine = Engine::new(
        ApplicationType::RPG,
        State {
            to_send_channel_messages: vec![],
            state_type: StateType::RPGShared(rpg_shared_state),
        },
        shared_state,
        scenes,
        web_socket_wrapper,
    );
    engine.init();
    engine
}
