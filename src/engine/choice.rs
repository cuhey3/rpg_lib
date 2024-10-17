use crate::rpg::mechanism::choice_kind::ChoiceKind;
use crate::rpg::mechanism::choice_kind::ChoiceKind::*;

#[derive(Debug)]
pub struct ChoiceTree {
    pub chose_kinds: Vec<ChoiceKind>,
    pub choice_list: Vec<Vec<String>>,
    pub now_choice: Choice,
    pub choice_indexes: Vec<usize>,
    pub root_choice: Choice,
}

impl ChoiceTree {
    pub fn get_now(&self) -> ChoiceKind {
        self.now_choice.own_token.clone()
    }

    pub fn choose(&mut self, index: usize) {
        self.choice_indexes.push(index);
        let mut rewritable_index = index;
        if let ItemInventory = &self.now_choice.own_token {
            rewritable_index = 0_usize;
        }
        if let Emote = &self.now_choice.own_token {
            rewritable_index = 0_usize;
        }
        if let ChoseNth(..) = &self.now_choice.own_token {
            rewritable_index = 0_usize;
        }
        if let Some(branch) = &mut self.now_choice.branch {
            if let Some(choice) = branch.get_mut(rewritable_index) {
                if let ChoseNth(token, ..) = &choice.own_token {
                    choice.own_token = ChoseNth(token.clone(), Some(index));
                }
                self.chose_kinds.push(self.now_choice.own_token.clone());
                self.now_choice = choice.clone();
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    pub fn undo(&mut self) {
        let indexes_len = self.choice_indexes.len();
        if indexes_len == 0 {
            return;
        }
        self.choice_indexes.remove(indexes_len - 1);
        let copied_choice_indexes = self.choice_indexes.clone();
        self.reset();
        for index in copied_choice_indexes.iter() {
            self.choose(*index);
        }
    }

    pub fn reset(&mut self) {
        self.now_choice = self.root_choice.clone();
        self.choice_indexes = vec![];
        self.chose_kinds = vec![];
        self.choice_list = vec![];
    }
}

pub struct ChoiceSetting {
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug)]
pub struct Choice {
    pub own_token: ChoiceKind,
    pub label: String,
    pub branch_description: Option<String>,
    pub branch: Option<Vec<Choice>>,
}

impl Choice {
    pub fn confirm_choice() -> Choice {
        Choice {
            label: Confirm.get_choice_string(),
            own_token: Confirm,
            branch_description: None,
            branch: Some(vec![
                Choice::no_choice_from_with_label(Decide, "はい".to_string()),
                Choice::no_choice_from_with_label(Undo, "いいえ".to_string()),
            ]),
        }
    }
    pub fn no_choice_from(own_token: ChoiceKind) -> Choice {
        Choice {
            label: own_token.get_choice_string(),
            own_token,
            branch_description: None,
            branch: None,
        }
    }
    fn no_choice_from_with_label(own_token: ChoiceKind, label: String) -> Choice {
        Choice {
            label,
            own_token,
            branch_description: None,
            branch: None,
        }
    }
    pub fn label_or_token_string(&self) -> String {
        if self.label.is_empty() {
            self.own_token.get_choice_string()
        } else {
            self.label.to_owned()
        }
    }
    pub fn get_branch_labels(&self) -> Vec<String> {
        if let Some(branch) = &self.branch {
            branch
                .iter()
                .map(|choice| choice.label_or_token_string())
                .collect()
        } else {
            vec![]
        }
    }
}
