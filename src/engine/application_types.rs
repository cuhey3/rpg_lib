use crate::rpg::battle::BattleState;
use crate::rpg::event::EventState;
use crate::rpg::field::FieldState;
use crate::rpg::menu::MenuState;
use crate::rpg::rpg_shared_state::RPGSharedState;
use crate::rpg::title::TitleState;

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
