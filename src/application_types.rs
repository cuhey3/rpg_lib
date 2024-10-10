use crate::rpg::RPGSharedState;

pub enum ApplicationType {
    RPG,
}

pub enum StateType {
    RPGShared(RPGSharedState),
}
