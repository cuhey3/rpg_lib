use crate::rpg::RPGSharedState;

pub struct Item {
    pub name: String,
    pub item_type: ItemType,
    pub consume_func: fn(&Item, &mut RPGSharedState),
    pub description: String,
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

        // TODO
        // RPGSharedState ではなく、Stateを受け取れるように変更(なんかすごい効果を持ったアイテムを実装できるように）
        fn consume_func(item: &Item, rpg_shared_state: &mut RPGSharedState) {
            match &item.item_type {
                ItemType::Consumable => match item.name.as_str() {
                    "薬草" => {
                        rpg_shared_state.characters[0].current_hp = rpg_shared_state.characters[0]
                            .max_hp
                            .min(rpg_shared_state.characters[0].current_hp + 30);
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
