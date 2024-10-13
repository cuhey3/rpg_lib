use crate::engine::application_types::SceneType::RPGMenu;
use crate::engine::application_types::StateType;
use crate::engine::scene::Scene;
use crate::engine::{Input, State};
use crate::rpg::item::{Item, ItemType};
use crate::rpg::RPGSharedState;
use crate::svg::animation::Animation;
use crate::svg::element_wrapper::ElementWrapper;
use crate::svg::{Cursor, SharedElements};
use wasm_bindgen_test::console_log;
use web_sys::Element;

struct MenuElements {
    inventory_container: ElementWrapper,
    inventories: Vec<Element>,
    inventory_confirm: ElementWrapper,
}

pub struct MenuState {
    inventory_opened: bool,
    inventory_confirm_opened: bool,
    elements: MenuElements,
    cursor: Cursor,
    inventory_cursor: Cursor,
    inventory_confirm_cursor: Cursor,
}

impl MenuState {
    pub fn create_menu_scene(shared_state: &mut State) -> Scene {
        let ref document = shared_state.elements.document;
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
        let inventory_confirm_cursor = Cursor::new(document, "inventory-confirm-cursor", 2, 50.0);
        let menu_state = MenuState {
            inventory_opened: false,
            inventory_confirm_opened: false,
            elements,
            cursor: Cursor::new(document, "menu-cursor", 5, 45.0),
            inventory_cursor: Cursor::new(document, "inventory-cursor", 1, 45.0),
            inventory_confirm_cursor,
        };
        let consume_func = menu_state.create_consume_func();
        let init_func = menu_state.create_init_func();
        let scene_type = RPGMenu(menu_state);
        Scene {
            element_id: "menu".to_string(),
            scene_type,
            consume_func,
            init_func,
        }
    }

    pub fn create_init_func(&self) -> fn(&mut Scene, &mut State) {
        fn init_func(_: &mut Scene, shared_state: &mut State) {
            shared_state.elements.menu_scene.show();
            console_log!("init end");
        }
        init_func
    }

    pub fn show_inventory(&mut self, elements: &mut SharedElements, items: &Vec<Item>) {
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
        let target_item = items.get(self.inventory_cursor.choose_index);
        if target_item.is_some() {
            elements.message.show();
            elements
                .document
                .get_element_by_id("message-1")
                .unwrap()
                .set_inner_html(target_item.unwrap().description.as_str());
        }
    }
    pub fn create_consume_func(&self) -> fn(&mut Scene, &mut State, Input) {
        fn consume_func(scene: &mut Scene, shared_state: &mut State, input: Input) {
            match &mut scene.scene_type {
                RPGMenu(menu_state) => {
                    let ref menu_elements = menu_state.elements;
                    if let State {
                        state_type: StateType::RPGShared(rpg_shared_state),
                        elements,
                        ..
                    } = shared_state
                    {
                        match input {
                            Input::ArrowUp | Input::ArrowDown => {
                                if menu_state.inventory_confirm_opened {
                                    menu_state.inventory_confirm_cursor.consume(input);
                                    return;
                                }
                                if !menu_state.inventory_opened {
                                    menu_state.cursor.consume(input);
                                    return;
                                }
                                let inventory_len = rpg_shared_state.characters[0].inventory.len();
                                if inventory_len < 2 {
                                    return;
                                }
                                menu_state
                                    .inventory_cursor
                                    .update_choice_length(inventory_len);
                                menu_state.inventory_cursor.consume(input);
                                shared_state.elements.message.show();
                                let target_item = rpg_shared_state.characters[0]
                                    .inventory
                                    .get(menu_state.inventory_cursor.choose_index);
                                if target_item.is_some() {
                                    shared_state
                                        .elements
                                        .document
                                        .get_element_by_id("message-1")
                                        .unwrap()
                                        .set_inner_html(target_item.unwrap().description.as_str());
                                }
                            }
                            Input::Enter => {
                                if menu_state.inventory_opened {
                                    if rpg_shared_state.characters[0].inventory.is_empty() {
                                        return;
                                    }
                                    if menu_state.inventory_confirm_opened {
                                        if menu_state.inventory_confirm_cursor.choose_index == 1 {
                                            let item_name = &rpg_shared_state.characters[0]
                                                .inventory
                                                [menu_state.inventory_cursor.choose_index]
                                                .name
                                                .to_owned();
                                            rpg_shared_state.characters[0]
                                                .inventory
                                                .remove(menu_state.inventory_cursor.choose_index);
                                            shared_state.interrupt_animations.push(vec![
                                                Animation::create_message(format!(
                                                    "{}を捨てた",
                                                    item_name
                                                )),
                                            ]);
                                            menu_state.inventory_confirm_cursor.reset();
                                            menu_state.inventory_cursor.reset();
                                            menu_state.inventory_confirm_opened = false;
                                            menu_elements.inventory_confirm.hide();
                                            if rpg_shared_state.characters[0].inventory.is_empty() {
                                                menu_state.inventory_opened = false;
                                                menu_elements.inventory_container.hide();
                                                return;
                                            }
                                            menu_state.show_inventory(
                                                elements,
                                                &rpg_shared_state.characters[0].inventory,
                                            );
                                            return;
                                        }
                                        match &rpg_shared_state.characters[0].inventory
                                            [menu_state.inventory_cursor.choose_index]
                                            .item_type
                                        {
                                            ItemType::Weapon => {
                                                shared_state.interrupt_animations.push(vec![
                                                    Animation::create_message(
                                                        "武器は使用できません".to_string(),
                                                    ),
                                                ]);
                                                return;
                                            }
                                            _ => {}
                                        }
                                        let item = Item::new(
                                            &rpg_shared_state.characters[0].inventory
                                                [menu_state.inventory_cursor.choose_index]
                                                .name,
                                        );
                                        let consume_func = item.consume_func;
                                        consume_func(&item, rpg_shared_state);
                                        rpg_shared_state.characters[0]
                                            .inventory
                                            .remove(menu_state.inventory_cursor.choose_index);
                                        menu_state.inventory_cursor.update_choice_length(
                                            rpg_shared_state.characters[0].inventory.len(),
                                        );
                                        menu_state.inventory_cursor.reset();
                                        menu_state.inventory_confirm_opened = false;
                                        menu_elements.inventory_confirm.hide();
                                        menu_state.show_inventory(
                                            elements,
                                            &rpg_shared_state.characters[0].inventory,
                                        );
                                        shared_state.interrupt_animations.push(vec![
                                            Animation::create_message(
                                                "薬草を使用しました。HPが30回復".to_string(),
                                            ),
                                        ]);
                                        return;
                                    } else {
                                        menu_state.inventory_confirm_opened = true;
                                        menu_elements.inventory_confirm.show();
                                        return;
                                    }
                                }
                                match menu_state.cursor.choose_index {
                                    0 => {
                                        let character = rpg_shared_state.characters.get(0).unwrap();
                                        if character.inventory.is_empty() {
                                            shared_state.interrupt_animations.push(vec![
                                                Animation::create_message(
                                                    "何も持っていない！".to_owned(),
                                                ),
                                            ]);
                                            return;
                                        }
                                        let inventory = &character.inventory;
                                        menu_state.show_inventory(elements, inventory);
                                    }
                                    2 => {
                                        let character = &rpg_shared_state.characters[0];
                                        console_log!(
                                            "character_u32, {},{}",
                                            character.current_hp,
                                            character.max_hp
                                        );
                                        console_log!(
                                            "treasure_box_usize, {:?}",
                                            rpg_shared_state.treasure_box_opened
                                        );
                                        console_log!(
                                            "map_usize, {}",
                                            shared_state.primitives.map_index
                                        );
                                        console_log!(
                                            "map_isize, {}, {}",
                                            rpg_shared_state.characters[0].position.x,
                                            rpg_shared_state.characters[0].position.y
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
                                        RPGSharedState::update_save_data(shared_state);
                                        shared_state.interrupt_animations.push(vec![
                                            Animation::create_message("セーブしました".to_string()),
                                        ]);
                                    }
                                    3 => {
                                        shared_state.primitives.requested_scene_index = 0;
                                        shared_state
                                            .interrupt_animations
                                            .push(vec![Animation::create_fade_out_in()]);
                                    }
                                    4 => shared_state.primitives.requested_scene_index -= 2,
                                    _ => {}
                                }
                            }

                            Input::Cancel => {
                                if menu_state.inventory_confirm_opened {
                                    menu_state.inventory_confirm_opened = false;
                                    menu_elements.inventory_confirm.hide();
                                    return;
                                }
                                if menu_state.inventory_opened {
                                    menu_elements.inventory_container.hide();
                                    menu_state.inventory_opened = false;
                                    shared_state.elements.message.hide();
                                    return;
                                }
                                shared_state.primitives.requested_scene_index -= 2;
                            }
                            _ => (),
                        }
                    }
                }
                _ => panic!(),
            }
        }
        consume_func
    }
}
