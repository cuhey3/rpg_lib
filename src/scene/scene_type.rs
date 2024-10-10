use crate::scene::battle::BattleStatus;
use crate::scene::field::FieldStatus;
use crate::scene::menu::MenuStatus;
use crate::scene::title::TitleStatus;

pub enum SceneType {
    Title(TitleStatus),
    Field(FieldStatus),
    Battle(BattleStatus),
    Menu(MenuStatus),
}
