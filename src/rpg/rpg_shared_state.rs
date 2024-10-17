use crate::engine::application_types::StateType;
use crate::engine::{PositionMessage, State};
use crate::rpg::{Character, SaveData};

pub struct RPGSharedState {
    pub treasure_box_opened: Vec<Vec<usize>>,
    pub save_data: SaveData,
    pub online_users: Vec<PositionMessage>,
    pub to_send_channel_messages: Vec<String>,
    pub characters: Vec<Character>,
}

impl RPGSharedState {
    pub fn update_save_data(shared_state: &mut State) {
        if let StateType::RPGShared(rpg_shared_state) = &mut shared_state.state_type {
            rpg_shared_state.save_data.update(
                &mut rpg_shared_state.characters,
                &rpg_shared_state.treasure_box_opened,
                shared_state.primitives.map_index,
            );
        }
    }
    pub fn load_save_data(shared_state: &mut State) {
        if let StateType::RPGShared(rpg_shared_state) = &mut shared_state.state_type {
            rpg_shared_state
                .save_data
                .load(&mut rpg_shared_state.characters, true);
            rpg_shared_state.treasure_box_opened =
                rpg_shared_state.save_data.treasure_box_usize.to_vec();
            shared_state.primitives.map_index =
                *rpg_shared_state.save_data.map_usize.get(0).unwrap();
            shared_state.primitives.requested_map_index =
                *rpg_shared_state.save_data.map_usize.get(0).unwrap();
        }
    }
    pub fn new_game(shared_state: &mut State) {
        if let StateType::RPGShared(rpg_shared_state) = &mut shared_state.state_type {
            let mut new_save_data = SaveData::empty();
            new_save_data.load(&mut rpg_shared_state.characters, false);
            rpg_shared_state.treasure_box_opened = new_save_data.treasure_box_usize.to_vec();
            shared_state.primitives.map_index = *new_save_data.map_usize.get(0).unwrap();
            shared_state.primitives.requested_map_index = *new_save_data.map_usize.get(0).unwrap();
        }
    }
}
