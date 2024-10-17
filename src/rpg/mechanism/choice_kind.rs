use crate::rpg::mechanism::choice_kind::ChoiceKind::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChoiceKind {
    Menu,
    UseItem,
    DropItem,
    Yes,
    No,
    Battle,
    Escape,
    Special,
    ItemInventory,
    Spell,
    Equip,
    Save,
    Title,
    CloseMenu,
    Emote,
    SendEmote,
    Chat,
    Nth(String),
    ChoseNth(String, Option<usize>),
    ItemOperation,
    Confirm,
    Decide,
    Undo,
    Root,
}

impl ChoiceKind {
    pub fn get_choice_string(&self) -> String {
        match self {
            Root => "",
            Menu => "",
            UseItem => "つかう",
            DropItem => "すてる",
            Yes => "はい",
            No => "いいえ",
            Battle => "たたかう",
            Escape => "にげる",
            Special => "とくぎ",
            ItemInventory => "どうぐ",
            Spell => "じゅもん",
            Equip => "そうび",
            Save => "セーブ",
            Title => "タイトル",
            CloseMenu => "とじる",
            Emote => "エモート",
            SendEmote => "",
            Chat => "チャット",
            Confirm => "",
            Undo => "",
            Decide => "",
            Nth(..) => "",
            ChoseNth(..) => "",
            ItemOperation => "",
        }
        .to_string()
    }
}
