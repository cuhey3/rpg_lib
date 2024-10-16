use crate::engine::application_types::SceneType::RPGMenu;
use crate::engine::application_types::StateType;
use crate::engine::scene::Scene;
use crate::engine::{
    ChoiceInstance, ChoiceSetting, ChoiceToken, EmoteMessage, Input, State, SvgRenderer,
};
use crate::rpg::item::{Item, ItemType};
use crate::rpg::RPGSharedState;
use crate::svg::animation::{Animation, AnimationSpan};
use crate::svg::CursorType;
use crate::Position;
use wasm_bindgen_test::console_log;

pub struct MenuState {
    choice_instance: ChoiceInstance,
    menu_renderer: SvgRenderer,
    inventory_renderer: SvgRenderer,
    inventory_confirm_renderer: SvgRenderer,
    common_confirm_renderer: SvgRenderer,
    emote_renderer: SvgRenderer,
    emotes: Vec<String>,
}

impl MenuState {
    pub fn create_menu_scene(_: &mut State) -> Scene {
        let choice_setting = ChoiceSetting::get_menu_setting();
        let choice_instance = choice_setting.get_menu_instance();
        let mut emotes = "👉👆👈👇👍🤨😆🤩🥺"
            .chars()
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        emotes.push("☺️".to_string());

        let menu_state = MenuState {
            choice_instance,
            menu_renderer: SvgRenderer::new("menu".to_string(), 45.0),
            inventory_renderer: SvgRenderer::new("menu-inventory".to_string(), 45.0),
            inventory_confirm_renderer: SvgRenderer::new(
                "menu-inventory-confirm".to_string(),
                50.0,
            ),
            common_confirm_renderer: SvgRenderer::new("menu-common-confirm".to_string(), 50.0),
            emote_renderer: SvgRenderer::new("menu-emote".to_string(), 45.0),
            emotes,
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
        fn init_func(scene: &mut Scene, shared_state: &mut State) {
            shared_state.elements.menu_scene.show();
            if let Scene {
                scene_type: RPGMenu(menu_state),
                ..
            } = scene
            {
                menu_state.choice_instance = ChoiceSetting::get_menu_setting().get_menu_instance();
                let labels = menu_state.choice_instance.now_choice.get_branch_labels();
                menu_state.menu_renderer.load();
                menu_state
                    .menu_renderer
                    .cursor
                    .update_choice_length(labels.len());
                menu_state.menu_renderer.render(labels, "");
            }
            console_log!("init end");
        }
        init_func
    }

    pub fn create_consume_func(&self) -> fn(&mut Scene, &mut State, Input) {
        fn consume_func(scene: &mut Scene, shared_state: &mut State, input: Input) {
            if let RPGMenu(menu_state) = &mut scene.scene_type {
                {
                    if let State {
                        state_type: StateType::RPGShared(rpg_shared_state),
                        to_send_channel_messages,
                        ..
                    } = shared_state
                    {
                        match input {
                            // 矢印キーは状態に応じてカーソルを動かすのみ
                            Input::ArrowUp
                            | Input::ArrowDown
                            | Input::ArrowRight
                            | Input::ArrowLeft => {
                                match menu_state.choice_instance.get_now() {
                                    ChoiceToken::Menu => {
                                        menu_state.menu_renderer.cursor.consume(input)
                                    }
                                    ChoiceToken::ItemInventory => {
                                        menu_state.inventory_renderer.cursor.consume(input)
                                    }
                                    ChoiceToken::ItemOperation => {
                                        menu_state.inventory_confirm_renderer.cursor.consume(input)
                                    }
                                    ChoiceToken::Confirm => {
                                        menu_state.common_confirm_renderer.cursor.consume(input)
                                    }
                                    ChoiceToken::Emote => {
                                        menu_state.emote_renderer.cursor.consume(input)
                                    }
                                    _ => {}
                                }
                                return;
                            }
                            Input::Enter => {
                                // 決定キーが押された場合、まず choice_instance の状態を先に進める
                                match menu_state.choice_instance.get_now() {
                                    ChoiceToken::Menu => {
                                        menu_state
                                            .choice_instance
                                            .choose(menu_state.menu_renderer.cursor.choose_index);
                                    }
                                    ChoiceToken::ItemInventory => {
                                        menu_state.choice_instance.choose(
                                            menu_state.inventory_renderer.cursor.choose_index,
                                        );
                                        menu_state.choice_instance.choose(
                                            menu_state.inventory_renderer.cursor.choose_index,
                                        );
                                    }
                                    ChoiceToken::ItemOperation => {
                                        menu_state.choice_instance.choose(
                                            menu_state
                                                .inventory_confirm_renderer
                                                .cursor
                                                .choose_index,
                                        );
                                    }
                                    ChoiceToken::Emote => {
                                        menu_state
                                            .choice_instance
                                            .choose(menu_state.emote_renderer.cursor.choose_index);
                                        menu_state
                                            .choice_instance
                                            .choose(menu_state.emote_renderer.cursor.choose_index);
                                    }
                                    ChoiceToken::Confirm => {
                                        menu_state.choice_instance.choose(
                                            menu_state.common_confirm_renderer.cursor.choose_index,
                                        );
                                    }
                                    _ => {
                                        // TODO
                                        // 何もしなくていいのか？
                                    }
                                }
                                // 先に進めた choice_instance の状態に応じて、画面を更新
                                // 後続処理がないなら return
                                match menu_state.choice_instance.get_now() {
                                    ChoiceToken::Close => {
                                        menu_state.menu_renderer.hide();
                                        menu_state.menu_renderer.cursor.reset();
                                        shared_state.primitives.requested_scene_index -= 2;
                                        return;
                                    }
                                    ChoiceToken::Emote => {
                                        menu_state.emote_renderer.load();
                                        menu_state
                                            .emote_renderer
                                            .cursor
                                            .update_choice_length(menu_state.emotes.len());
                                        menu_state.emote_renderer.cursor.set_box_length(5, 2);
                                        menu_state
                                            .emote_renderer
                                            .cursor
                                            .set_cursor_type(CursorType::Box);
                                        menu_state
                                            .emote_renderer
                                            .render(menu_state.emotes.clone(), "");
                                        return;
                                    }
                                    // インベントリを開いて完了
                                    ChoiceToken::ItemInventory => {
                                        menu_state.inventory_renderer.load();

                                        // 何もアイテム持っていない時は続行させない
                                        if rpg_shared_state.characters[0].inventory.is_empty() {
                                            menu_state.choice_instance.undo();
                                            shared_state.interrupt_animations.push(vec![
                                                Animation::create_message(
                                                    "何も持っていない！".to_string(),
                                                ),
                                            ]);
                                        } else {
                                            let item_names = rpg_shared_state.characters[0]
                                                .inventory
                                                .iter()
                                                .map(|i| i.name.clone())
                                                .collect::<Vec<String>>();
                                            menu_state
                                                .inventory_renderer
                                                .cursor
                                                .update_choice_length(item_names.len());
                                            menu_state.inventory_renderer.render(item_names, "");
                                        }
                                        return;
                                    }
                                    ChoiceToken::ItemOperation => {
                                        menu_state.inventory_confirm_renderer.load();
                                        let labels = menu_state
                                            .choice_instance
                                            .now_choice
                                            .get_branch_labels();
                                        menu_state
                                            .inventory_confirm_renderer
                                            .cursor
                                            .update_choice_length(labels.len());
                                        menu_state.inventory_confirm_renderer.render(labels, "");
                                    }
                                    ChoiceToken::Equip | ChoiceToken::Chat => {
                                        menu_state.choice_instance.undo();
                                    }
                                    ChoiceToken::UseItem => {
                                        for token in
                                            menu_state.choice_instance.choice_tokens.clone().iter()
                                        {
                                            if let ChoiceToken::NthChoose(_, index) = token {
                                                let index = index.unwrap();
                                                match &rpg_shared_state.characters[0].inventory
                                                    [index]
                                                    .item_type
                                                {
                                                    ItemType::Weapon => {
                                                        shared_state.interrupt_animations.push(
                                                            vec![Animation::create_message(
                                                                "武器は使用できません".to_string(),
                                                            )],
                                                        );
                                                        menu_state.choice_instance.undo();
                                                        return;
                                                    }
                                                    _ => {}
                                                }
                                                let item = Item::new(
                                                    &rpg_shared_state.characters[0].inventory
                                                        [index]
                                                        .name,
                                                );
                                                let consume_func = item.consume_func;
                                                consume_func(&item, rpg_shared_state);
                                                shared_state.interrupt_animations.push(vec![
                                                    Animation::create_message(
                                                        "薬草を使用しました。HPが30回復"
                                                            .to_string(),
                                                    ),
                                                ]);
                                                rpg_shared_state.characters[0]
                                                    .inventory
                                                    .remove(index);
                                                menu_state.choice_instance.undo();
                                            }
                                        }
                                        menu_state.inventory_confirm_renderer.hide();
                                        menu_state.inventory_confirm_renderer.cursor.reset();
                                        menu_state.inventory_renderer.cursor.reset();
                                        menu_state.inventory_renderer.load();
                                        menu_state.choice_instance.undo();
                                        menu_state.choice_instance.undo();
                                        // 何もアイテム持っていない時は続行させない
                                        if rpg_shared_state.characters[0].inventory.is_empty() {
                                            menu_state.inventory_renderer.hide();
                                            menu_state.choice_instance.undo();
                                        } else {
                                            let item_names = rpg_shared_state.characters[0]
                                                .inventory
                                                .iter()
                                                .map(|i| i.name.clone())
                                                .collect::<Vec<String>>();
                                            menu_state
                                                .inventory_renderer
                                                .cursor
                                                .update_choice_length(item_names.len());
                                            menu_state.inventory_renderer.render(item_names, "");
                                        }
                                        return;
                                    }
                                    ChoiceToken::SendEmote => {
                                        for token in menu_state.choice_instance.choice_tokens.iter()
                                        {
                                            if let ChoiceToken::NthChoose(_, index) = token {
                                                let index = index.unwrap();
                                                let emote = menu_state.emotes[index].clone();
                                                let Position { x, y } =
                                                    rpg_shared_state.characters[0].position;

                                                let message = EmoteMessage {
                                                    user_name: shared_state.user_name.to_owned(),
                                                    position_x: x,
                                                    position_y: y,
                                                    map_index: shared_state.primitives.map_index,
                                                    emote,
                                                };
                                                to_send_channel_messages
                                                    .push(serde_json::to_string(&message).unwrap());
                                            }
                                        }
                                        menu_state.choice_instance.undo();
                                        menu_state.choice_instance.undo();
                                        return;
                                    }
                                    ChoiceToken::Save
                                    | ChoiceToken::Title
                                    | ChoiceToken::DropItem => {
                                        // Confirm 要素を準備
                                        let description = menu_state
                                            .choice_instance
                                            .now_choice
                                            .branch_description
                                            .clone()
                                            .unwrap();
                                        menu_state.choice_instance.choose(0);
                                        menu_state.common_confirm_renderer.load();
                                        let labels = menu_state
                                            .choice_instance
                                            .now_choice
                                            .get_branch_labels();
                                        menu_state
                                            .common_confirm_renderer
                                            .cursor
                                            .update_choice_length(labels.len());
                                        menu_state
                                            .common_confirm_renderer
                                            .render(labels, description.as_str());
                                        return;
                                    }
                                    // choice_instance 巻き戻し（Confirmを必要とした要素まで）、Confirm 要素を隠す
                                    ChoiceToken::Undo => {
                                        menu_state.choice_instance.undo();
                                        menu_state.choice_instance.undo();
                                        menu_state.choice_instance.undo();
                                        menu_state.common_confirm_renderer.hide();
                                        menu_state.common_confirm_renderer.cursor.reset();
                                        return;
                                    }
                                    // choice_instance 巻き戻し（Confirmを必要とした要素まで）、Confirm 要素を隠す
                                    ChoiceToken::Decide => {
                                        menu_state.choice_instance.undo();
                                        menu_state.choice_instance.undo();
                                        menu_state.common_confirm_renderer.hide();
                                        menu_state.common_confirm_renderer.cursor.reset();
                                    }
                                    _ => {}
                                }
                                // Confirm を必要とした要素についてはここでさらに後続処理
                                match menu_state.choice_instance.get_now() {
                                    ChoiceToken::Save => {
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
                                        menu_state.choice_instance.undo();
                                        return;
                                    }
                                    ChoiceToken::Title => {
                                        menu_state.menu_renderer.hide();
                                        menu_state.menu_renderer.cursor.reset();
                                        menu_state.inventory_renderer.hide();
                                        menu_state.inventory_renderer.cursor.reset();
                                        menu_state.common_confirm_renderer.hide();
                                        menu_state.common_confirm_renderer.cursor.reset();
                                        shared_state.primitives.requested_scene_index = 0;
                                        shared_state.interrupt_animations.push(vec![
                                            Animation::create_fade_out_in_with_span(
                                                AnimationSpan::FadeOutInMedium,
                                            ),
                                        ]);
                                        return;
                                    }
                                    ChoiceToken::DropItem => {
                                        for token in menu_state.choice_instance.choice_tokens.iter()
                                        {
                                            if let ChoiceToken::NthChoose(_, index) = token {
                                                let index = index.unwrap();
                                                let item_name = &rpg_shared_state.characters[0]
                                                    .inventory[index]
                                                    .name
                                                    .to_owned();
                                                rpg_shared_state.characters[0]
                                                    .inventory
                                                    .remove(index);
                                                shared_state.interrupt_animations.push(vec![
                                                    Animation::create_message(format!(
                                                        "{}を捨てた",
                                                        item_name
                                                    )),
                                                ]);
                                            }
                                        }
                                        menu_state.inventory_confirm_renderer.hide();
                                        menu_state.inventory_confirm_renderer.cursor.reset();
                                        menu_state.inventory_renderer.cursor.reset();
                                        menu_state.inventory_renderer.load();
                                        menu_state.choice_instance.undo();
                                        menu_state.choice_instance.undo();
                                        menu_state.choice_instance.undo();
                                        // 何もアイテム持っていない時は続行させない
                                        if rpg_shared_state.characters[0].inventory.is_empty() {
                                            menu_state.inventory_renderer.hide();
                                            menu_state.choice_instance.undo();
                                        } else {
                                            let item_names = rpg_shared_state.characters[0]
                                                .inventory
                                                .iter()
                                                .map(|i| i.name.clone())
                                                .collect::<Vec<String>>();
                                            menu_state
                                                .inventory_renderer
                                                .cursor
                                                .update_choice_length(item_names.len());
                                            menu_state.inventory_renderer.render(item_names, "");
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            Input::Cancel => {
                                match menu_state.choice_instance.get_now() {
                                    ChoiceToken::Menu => {
                                        menu_state.menu_renderer.hide();
                                        menu_state.menu_renderer.cursor.reset();
                                        menu_state.inventory_renderer.hide();
                                        menu_state.inventory_renderer.cursor.reset();
                                        menu_state.emote_renderer.hide();
                                        menu_state.emote_renderer.cursor.reset();
                                        menu_state.common_confirm_renderer.hide();
                                        menu_state.common_confirm_renderer.cursor.reset();
                                        shared_state.primitives.requested_scene_index -= 2;
                                        return;
                                    }
                                    ChoiceToken::ItemInventory => {
                                        menu_state.inventory_renderer.hide();
                                        menu_state.inventory_renderer.cursor.reset();
                                    }
                                    ChoiceToken::ItemOperation => {
                                        menu_state.inventory_confirm_renderer.hide();
                                        menu_state.inventory_confirm_renderer.cursor.reset();
                                        menu_state.choice_instance.undo();
                                    }
                                    ChoiceToken::Confirm => {
                                        menu_state.common_confirm_renderer.cursor.reset();
                                        menu_state.common_confirm_renderer.hide();
                                        menu_state.choice_instance.undo();
                                    }
                                    ChoiceToken::Emote => {
                                        menu_state.emote_renderer.hide();
                                        menu_state.emote_renderer.cursor.reset();
                                    }
                                    _ => {}
                                }
                                menu_state.choice_instance.undo();
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
        consume_func
    }
}
