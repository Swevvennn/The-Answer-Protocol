use std::fmt;

pub enum Error {
    NameInUse,
    NoExit,
    NotInGroup,
    AlreadyInGroup,
    ItemNotFound,
    ItemNotInInventory,
    NPCNotFound,
    NPCNotHostile,
    NoQuestAvailable,
    ConnectionFailed,
    SendFailed,
}

impl Error {
    pub fn code(&self) -> u16 {
        match self {
            Self::NameInUse => 201,
            Self::NoExit => 301,
            Self::NotInGroup => 401,
            Self::AlreadyInGroup => 402,
            Self::ItemNotFound => 404,
            Self::ItemNotInInventory => 404,
            Self::NPCNotFound => 404,
            Self::NPCNotHostile => 405,
            Self::NoQuestAvailable => 406,
            Self::ConnectionFailed => 900,
            Self::SendFailed => 901,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::NameInUse => "NAME_IN_USE",
            Self::NoExit => "NO_EXIT",
            Self::NotInGroup => "NOT_IN_GROUP",
            Self::AlreadyInGroup => "ALREADY_IN_GROUP",
            Self::ItemNotFound => "ITEM_NOT_FOUND",
            Self::ItemNotInInventory => "ITEM_NOT_IN_INVENTORY",
            Self::NPCNotFound => "NPC_NOT_FOUND",
            Self::NPCNotHostile => "NPC_NOT_HOSTILE",
            Self::NoQuestAvailable => "NO_QUEST_AVAILABLE",
            Self::ConnectionFailed => "CONNECTION_FAILED",
            Self::SendFailed => "SEND_FAILED",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ERR {} {}",
            self.code(),
            self.message()
        )
    }
}
