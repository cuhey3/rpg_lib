use crate::engine::application_types::SceneType::RPGMenu;
use crate::engine::application_types::StateType;
use crate::engine::scene::Scene;
use crate::engine::{ChoiceSetting, EmoteMessage, Input, State};
use crate::rpg::item::{Item, ItemType};
use crate::rpg::ChoiceKind::*;
use crate::rpg::RPGSharedState;
use crate::svg::animation::{Animation, AnimationSpan};
use crate::svg::Position;
use crate::svg::{RendererController, SvgRenderer};
use wasm_bindgen_test::console_log;

pub struct MenuState {
    renderer_controller: RendererController,
    emotes: Vec<String>,
}

impl MenuState {
    pub fn create_menu_scene(_: &mut State) -> Scene {
        let choice_setting = ChoiceSetting::get_menu_setting();
        let mut emotes = "👉👆👈👇👍🤨😆🤩🥺"
            .chars()
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        emotes.push("☺️".to_string());
        let mut emote_renderer = SvgRenderer::new(Emote, "menu-emote".to_string(), 45.0);
        emote_renderer.cursor.set_box_length(5, 2);
        let menu_state = MenuState {
            renderer_controller: RendererController {
                choice_tree: choice_setting.get_menu_choice_tree(),
                choice_setting,
                confirm_index: Some(3),
                renderers: vec![
                    SvgRenderer::new(Menu, "menu".to_string(), 45.0),
                    SvgRenderer::new(ItemInventory, "menu-inventory".to_string(), 45.0),
                    SvgRenderer::new(ItemOperation, "menu-item-operation".to_string(), 50.0),
                    SvgRenderer::new(Confirm, "menu-common-confirm".to_string(), 50.0),
                    emote_renderer,
                ],
            },
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
                menu_state.renderer_controller.initial_render();
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
                                menu_state.renderer_controller.delegate_input(input);
                                return;
                            }
                            Input::Enter => {
                                // 決定キーが押された場合、まず choice_tree の状態を先に進める
                                menu_state.renderer_controller.delegate_enter();

                                // 先に進めた choice_tree の状態に応じて、画面を更新
                                // 後続処理がないなら return
                                match menu_state.renderer_controller.now_choice_kind() {
                                    CloseMenu => {
                                        menu_state.renderer_controller.renderers[0].hide();
                                        menu_state.renderer_controller.renderers[0].cursor.reset();
                                        shared_state.primitives.requested_scene_index -= 2;
                                        return;
                                    }
                                    Emote => {
                                        menu_state
                                            .renderer_controller
                                            .render_with(menu_state.emotes.clone(), "");
                                        return;
                                    }
                                    // インベントリを開いて完了
                                    ItemInventory => {
                                        // 何もアイテム持っていない時は続行させない（描画しない）
                                        if rpg_shared_state.characters[0].inventory.is_empty() {
                                            menu_state.renderer_controller.undo_choice_tree();
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
                                                .renderer_controller
                                                .render_with(item_names, "");
                                        }
                                        return;
                                    }
                                    ItemOperation => {
                                        let labels = menu_state
                                            .renderer_controller
                                            .choice_tree
                                            .now_choice
                                            .get_branch_labels();
                                        menu_state.renderer_controller.render_with(labels, "");
                                    }
                                    Equip | Chat => {
                                        shared_state.interrupt_animations.push(vec![
                                            Animation::create_message("Coming soon...".to_string()),
                                        ]);
                                        menu_state.renderer_controller.undo_choice_tree();
                                    }
                                    UseItem => {
                                        let index = menu_state.renderer_controller.get_chose_nth();
                                        if index.is_none() {
                                            return;
                                        }
                                        let index = index.unwrap();
                                        match &rpg_shared_state.characters[0].inventory[index]
                                            .item_type
                                        {
                                            ItemType::Weapon => {
                                                shared_state.interrupt_animations.push(vec![
                                                    Animation::create_message(
                                                        "武器は使用できません".to_string(),
                                                    ),
                                                ]);
                                                menu_state.renderer_controller.undo_choice_tree();
                                                return;
                                            }
                                            _ => {}
                                        }
                                        let item = Item::new(
                                            &rpg_shared_state.characters[0].inventory[index].name,
                                        );
                                        let consume_func = item.consume_func;
                                        consume_func(&item, rpg_shared_state);
                                        shared_state.interrupt_animations.push(vec![
                                            Animation::create_message(
                                                "薬草を使用しました。HPが30回復".to_string(),
                                            ),
                                        ]);
                                        rpg_shared_state.characters[0].inventory.remove(index);
                                        menu_state.renderer_controller.undo_choice_tree();
                                        menu_state.renderer_controller.delegate_close();
                                        menu_state.renderer_controller.undo_choice_tree();
                                        // 何もアイテム持っていない時は続行させない
                                        if rpg_shared_state.characters[0].inventory.is_empty() {
                                            menu_state.renderer_controller.delegate_close();
                                        } else {
                                            let item_names = rpg_shared_state.characters[0]
                                                .inventory
                                                .iter()
                                                .map(|i| i.name.clone())
                                                .collect::<Vec<String>>();
                                            menu_state
                                                .renderer_controller
                                                .render_with(item_names, "");
                                        }
                                        return;
                                    }
                                    SendEmote => {
                                        let index = menu_state.renderer_controller.get_chose_nth();
                                        if index.is_none() {
                                            return;
                                        }
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
                                        menu_state.renderer_controller.undo_choice_tree();
                                        menu_state.renderer_controller.undo_choice_tree();
                                        return;
                                    }
                                    Save | Title | DropItem => {
                                        // Confirm 要素を準備
                                        menu_state.renderer_controller.delegate_confirm();
                                        return;
                                    }
                                    // choice_tree 巻き戻し（Confirmを必要とした要素まで）、Confirm 要素を隠す
                                    Undo => {
                                        menu_state.renderer_controller.undo_choice_tree();
                                        menu_state.renderer_controller.delegate_close();
                                        menu_state.renderer_controller.undo_choice_tree();
                                        return;
                                    }
                                    // choice_tree 巻き戻し（Confirmを必要とした要素まで）、Confirm 要素を隠す
                                    Decide => {
                                        // TODO
                                        // undo, delegate_close, undo が気持ち悪い
                                        menu_state.renderer_controller.undo_choice_tree();
                                        menu_state.renderer_controller.delegate_close();
                                    }
                                    _ => {}
                                }
                                // Confirm を必要とした要素についてはここでさらに後続処理
                                match menu_state.renderer_controller.now_choice_kind() {
                                    Save => {
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
                                        menu_state.renderer_controller.undo_choice_tree();
                                        return;
                                    }
                                    Title => {
                                        menu_state.renderer_controller.close_all();
                                        shared_state.primitives.requested_scene_index = 0;
                                        shared_state.interrupt_animations.push(vec![
                                            Animation::create_fade_out_in_with_span(
                                                AnimationSpan::FadeOutInMedium,
                                            ),
                                        ]);
                                        return;
                                    }
                                    DropItem => {
                                        let index = menu_state.renderer_controller.get_chose_nth();
                                        if index.is_none() {
                                            return;
                                        }
                                        let index = index.unwrap();
                                        let item_name = &rpg_shared_state.characters[0].inventory
                                            [index]
                                            .name
                                            .to_owned();
                                        rpg_shared_state.characters[0].inventory.remove(index);
                                        shared_state.interrupt_animations.push(vec![
                                            Animation::create_message(format!(
                                                "{}を捨てた",
                                                item_name
                                            )),
                                        ]);
                                        menu_state.renderer_controller.undo_choice_tree();
                                        menu_state.renderer_controller.delegate_close();
                                        menu_state.renderer_controller.undo_choice_tree();
                                        // 何もアイテム持っていない時は続行させない
                                        if rpg_shared_state.characters[0].inventory.is_empty() {
                                            menu_state.renderer_controller.delegate_close();
                                        } else {
                                            let item_names = rpg_shared_state.characters[0]
                                                .inventory
                                                .iter()
                                                .map(|i| i.name.clone())
                                                .collect::<Vec<String>>();
                                            menu_state
                                                .renderer_controller
                                                .render_with(item_names, "");
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            Input::Cancel => {
                                menu_state.renderer_controller.delegate_close();
                                match menu_state.renderer_controller.now_choice_kind() {
                                    Menu => {
                                        shared_state.primitives.requested_scene_index -= 2;
                                        return;
                                    }
                                    ItemOperation | Confirm => {
                                        // 追加で undo
                                        // なぜ？
                                        menu_state.renderer_controller.undo_choice_tree();
                                    }
                                    _ => {}
                                }
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
