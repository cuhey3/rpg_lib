use crate::animation::Animation;
use crate::scene::scene_type::SceneType::Menu;
use crate::scene::Scene;
use crate::{Character, Cursor, ElementWrapper, Item, ItemType, SaveData, SharedStatus};
use wasm_bindgen_test::console_log;
use web_sys::Element;

struct MenuElements {
    inventory_container: ElementWrapper,
    inventories: Vec<Element>,
    inventory_confirm: ElementWrapper,
}

pub struct MenuStatus {
    inventory_opened: bool,
    inventory_confirm_opened: bool,
    inventory_choose_index: usize,
    elements: MenuElements,
    cursor: Cursor,
    inventory_cursor: Cursor,
}

impl MenuStatus {
    pub fn create_init_func(&self) -> fn(&mut Scene, &mut SharedStatus, &mut Vec<Character>) {
        fn init_func(_: &mut Scene, shared_status: &mut SharedStatus, _: &mut Vec<Character>) {
            shared_status.elements.menu_scene.show();
            console_log!("init end");
        }
        init_func
    }
    pub fn show_inventory(&mut self, shared_status: &mut SharedStatus, items: &Vec<Item>) {
        self.elements
            .inventories
            .iter()
            .for_each(|element| element.set_inner_html(""));
        for index in 0..items.len() {
            self.elements.inventories[index]
                .set_inner_html(items.get(index).unwrap().name.as_str());
        }
        self.elements.inventory_container.show();
        self.inventory_opened = true;
        let target_item = items.get(self.inventory_choose_index);
        if target_item.is_some() {
            shared_status.elements.message.show();
            shared_status
                .elements
                .document
                .get_element_by_id("message-1")
                .unwrap()
                .set_inner_html(target_item.unwrap().description.as_str());
        }
    }
    pub fn create_consume_func(
        &self,
    ) -> fn(&mut Scene, &mut SharedStatus, &mut Vec<Character>, String) {
        fn consume_func(
            scene: &mut Scene,
            shared_status: &mut SharedStatus,
            characters: &mut Vec<Character>,
            key: String,
        ) {
            match &mut scene.scene_type {
                Menu(menu_status) => {
                    let ref elements = menu_status.elements;
                    match key.as_str() {
                        "ArrowUp" | "ArrowDown" => {
                            if !menu_status.inventory_opened {
                                menu_status.cursor.consume(key);
                                return;
                            }
                            let inventory_len = characters[0].inventory.len();
                            if inventory_len < 2 {
                                return;
                            }
                            menu_status
                                .inventory_cursor
                                .update_choice_length(inventory_len);
                            menu_status.inventory_cursor.consume(key);
                            shared_status.elements.message.show();
                            let target_item = characters[0]
                                .inventory
                                .get(menu_status.inventory_cursor.choose_index);
                            if target_item.is_some() {
                                shared_status
                                    .elements
                                    .document
                                    .get_element_by_id("message-1")
                                    .unwrap()
                                    .set_inner_html(target_item.unwrap().description.as_str());
                            }
                        }
                        "a" => {
                            if menu_status.inventory_opened {
                                if characters[0].inventory.is_empty() {
                                    return;
                                }
                                if menu_status.inventory_confirm_opened {
                                    shared_status.has_message = true;
                                    shared_status.interrupt_animations.push(vec![
                                        Animation::create_message(
                                            "薬草を使用しました。HPが30回復".to_string(),
                                        ),
                                    ]);
                                    let item = Item::new(
                                        &characters[0].inventory
                                            [menu_status.inventory_choose_index]
                                            .name,
                                    );
                                    let consume_func = item.consume_func;
                                    consume_func(&item, shared_status, characters);
                                    characters[0]
                                        .inventory
                                        .remove(menu_status.inventory_choose_index);
                                    menu_status
                                        .inventory_cursor
                                        .update_choice_length(characters[0].inventory.len());
                                    menu_status.inventory_cursor.reset();
                                    menu_status.inventory_confirm_opened = false;
                                    elements.inventory_confirm.hide();
                                    menu_status
                                        .show_inventory(shared_status, &characters[0].inventory);
                                    return;
                                } else {
                                    match &characters[0].inventory
                                        [menu_status.inventory_choose_index]
                                        .item_type
                                    {
                                        ItemType::Consumable => {
                                            menu_status.inventory_confirm_opened = true;
                                            elements.inventory_confirm.show();
                                            return;
                                        }
                                        ItemType::Weapon => {
                                            shared_status.has_message = true;
                                            shared_status.interrupt_animations.push(vec![
                                                Animation::create_message(
                                                    "武器は使用できません".to_string(),
                                                ),
                                            ]);
                                        }
                                    }
                                }
                            }
                            match menu_status.cursor.choose_index {
                                0 => {
                                    let character = characters.get_mut(0).unwrap();
                                    let inventory = &character.inventory;
                                    menu_status.show_inventory(shared_status, inventory);
                                }
                                2 => {
                                    let character = &characters[0];
                                    console_log!(
                                        "character_u32, {},{}",
                                        character.current_hp,
                                        character.max_hp
                                    );
                                    console_log!(
                                        "treasure_box_usize, {:?}",
                                        shared_status.treasure_box_opened
                                    );
                                    console_log!("map_usize, {}", shared_status.map_index);
                                    console_log!(
                                        "map_isize, {}, {}",
                                        characters[0].position.x,
                                        characters[0].position.y
                                    );
                                    console_log!(
                                        "inventory_string, {}",
                                        character
                                            .inventory
                                            .iter()
                                            .map(|item| item.name.clone())
                                            .collect::<Vec<String>>()
                                            .join(",")
                                    );
                                    shared_status.update_save_data(characters);
                                    shared_status.has_message = true;
                                    shared_status.interrupt_animations.push(vec![
                                        Animation::create_message("セーブしました".to_string()),
                                    ]);
                                }
                                3 => {
                                    shared_status.requested_scene_index -= 3;
                                    shared_status
                                        .interrupt_animations
                                        .push(vec![Animation::create_fade_out_in()]);
                                }
                                4 => shared_status.requested_scene_index -= 2,
                                _ => {}
                            }
                        }

                        "Escape" => {
                            if menu_status.inventory_confirm_opened {
                                menu_status.inventory_confirm_opened = false;
                                elements.inventory_confirm.hide();
                                return;
                            }
                            if menu_status.inventory_opened {
                                elements.inventory_container.hide();
                                menu_status.inventory_opened = false;
                                shared_status.elements.message.hide();
                                return;
                            }
                            shared_status.requested_scene_index -= 2;
                        }
                        _ => (),
                    }
                }
                _ => panic!(),
            }
        }
        consume_func
    }
}
pub fn create_menu_scene(shared_status: &mut SharedStatus) -> Scene {
    let ref document = shared_status.elements.document;
    let inventory_container =
        ElementWrapper::new(document.query_selector("#inventory").unwrap().unwrap());
    let inventory1 = document.query_selector("#inventory-1").unwrap().unwrap();
    let inventory2 = document.query_selector("#inventory-2").unwrap().unwrap();
    let inventory_confirm = ElementWrapper::new(
        document
            .query_selector("#inventory-confirm")
            .unwrap()
            .unwrap(),
    );
    let elements = MenuElements {
        inventory_container,
        inventories: vec![inventory1, inventory2],
        inventory_confirm,
    };
    let menu_status = MenuStatus {
        inventory_opened: false,
        inventory_confirm_opened: false,
        inventory_choose_index: 0,
        elements,
        cursor: Cursor::new(document, "menu-cursor", 5, 45.0),
        inventory_cursor: Cursor::new(document, "inventory-cursor", 1, 45.0),
    };
    let consume_func = menu_status.create_consume_func();
    let init_func = menu_status.create_init_func();
    let scene_type = Menu(menu_status);
    Scene {
        element_id: "menu".to_string(),
        scene_type,
        consume_func,
        init_func,
    }
}
