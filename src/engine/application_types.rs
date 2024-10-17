use crate::rpg::scenes::battle::BattleState;
use crate::rpg::scenes::event::EventState;
use crate::rpg::scenes::field::FieldState;
use crate::rpg::scenes::menu::MenuState;
use crate::rpg::scenes::title::TitleState;
use crate::rpg::state::rpg_shared_state::RPGSharedState;

pub enum StateType {
    RPGShared(RPGSharedState),
    TBDStateType,
}

pub enum SceneType {
    RPGTitle(TitleState),
    RPGEvent(EventState),
    RPGField(FieldState),
    RPGBattle(BattleState),
    RPGMenu(MenuState),
}
