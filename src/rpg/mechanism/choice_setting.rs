use crate::engine::choice::{Choice, ChoiceSetting, ChoiceTree};
use crate::rpg::mechanism::choice_kind::ChoiceKind::*;

impl ChoiceSetting {
    fn new() -> ChoiceSetting {
        ChoiceSetting { choices: vec![] }
    }
    fn add_choices(&mut self, choice: &mut Vec<Choice>) -> &mut ChoiceSetting {
        self.choices.append(choice);
        self
    }

    pub fn get_battle_choice_tree(&self) -> ChoiceTree {
        let root_choice = Choice {
            own_token: Root,
            label: Root.get_choice_string(),
            branch_description: None,
            branch: Some(self.choices.clone()),
        };
        ChoiceTree {
            chose_kinds: vec![],
            choice_list: vec![],
            choice_indexes: vec![],
            now_choice: root_choice.clone(),
            root_choice,
        }
    }

    pub fn get_menu_choice_tree(&self) -> ChoiceTree {
        let root_choice = Choice {
            own_token: Menu,
            label: Menu.get_choice_string(),
            branch_description: None,
            branch: Some(self.choices.clone()),
        };
        ChoiceTree {
            chose_kinds: vec![],
            choice_list: vec![],
            choice_indexes: vec![],
            now_choice: root_choice.clone(),
            root_choice,
        }
    }
    pub fn get_battle_setting() -> ChoiceSetting {
        let mut setting = ChoiceSetting::new();
        setting.add_choices(&mut vec![
            Choice::no_choice_from(Battle),
            Choice::no_choice_from(Escape),
        ]);
        setting
    }
    pub fn get_menu_setting() -> ChoiceSetting {
        let use_choice = Choice::no_choice_from(UseItem);
        let drop_choice = Choice {
            own_token: DropItem,
            label: DropItem.get_choice_string(),
            branch_description: Some("本当に捨てますか？".to_string()),
            branch: Some(vec![Choice::confirm_choice()]),
        };
        let mut setting = ChoiceSetting::new();
        setting.add_choices(&mut vec![
            Choice {
                own_token: ItemInventory,
                label: "".to_string(),
                branch_description: None,
                branch: Some(vec![Choice {
                    own_token: ChoseNth("Item".to_string(), None),
                    label: "".to_string(),
                    branch_description: None,
                    branch: Some(vec![Choice {
                        own_token: ItemOperation,
                        label: "".to_string(),
                        branch_description: None,
                        branch: Some(vec![use_choice.clone(), drop_choice.clone()]),
                    }]),
                }]),
            },
            Choice::no_choice_from(Equip),
            Choice {
                own_token: Emote,
                label: "".to_string(),
                branch_description: None,
                branch: Some(vec![Choice {
                    own_token: ChoseNth("Emote".to_string(), None),
                    label: "".to_string(),
                    branch_description: None,
                    branch: Some(vec![Choice::no_choice_from(SendEmote)]),
                }]),
            },
            Choice::no_choice_from(Chat),
            Choice {
                own_token: Save,
                label: "".to_string(),
                branch_description: Some("セーブを上書きします。よろしいですか？".to_string()),
                branch: Some(vec![Choice::confirm_choice()]),
            },
            Choice {
                own_token: Title,
                label: "".to_string(),
                branch_description: Some("タイトルに戻ります。よろしいですか？".to_string()),
                branch: Some(vec![Choice::confirm_choice()]),
            },
            Choice::no_choice_from(CloseMenu),
        ]);
        setting
    }
}
