use strum::IntoEnumIterator;

#[derive(Clone, strum_macros::EnumIter)]
pub enum Error {
    UnknownError,
    NotACommand,
    UnknownCommand,
    InvalidArguments,
    AlreadyAuthenticated,
    NotAuthenticated,
    NameInUse,
    InvalidName,
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
    ConnectionClosed,
    UnexpectedServerResponse,
    ServerTimeOut,
    ServerError,
}

impl Error {
    pub fn code(&self) -> u16 {
        match self {
            Self::UnknownError => 1,
            Self::NotACommand => 100,
            Self::UnknownCommand => 101,
            Self::InvalidArguments => 102,
            Self::AlreadyAuthenticated => 103,
            Self::NotAuthenticated => 104,
            Self::NameInUse => 201,
            Self::InvalidName => 202,
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
            Self::ConnectionClosed => 902,
            Self::UnexpectedServerResponse => 903,
            Self::ServerTimeOut => 904,
            Self::ServerError => 905,
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::UnknownError => "UNKNOWN_ERROR",
            Self::NotACommand => "NOT_A_COMMAND",
            Self::UnknownCommand => "UNKNOWN_COMMAND",
            Self::InvalidArguments => "INVALID_ARGUMENTS",
            Self::AlreadyAuthenticated => "ALREADY_AUTHENTICATED",
            Self::NotAuthenticated => "NOT_AUTHENTICATED",
            Self::NameInUse => "NAME_IN_USE",
            Self::InvalidName => "INVALID_NAME",
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
            Self::ConnectionClosed => "CONNECTION_CLOSED",
            Self::UnexpectedServerResponse => "UNEXPECTED_SERVER_RESPONSE",
            Self::ServerTimeOut => "SERVER_TIME_OUT",
            Self::ServerError => "SERVER_ERROR",
        }
    }

    pub fn from_str(s: &str) -> Result<Error, std::io::Error> {
        if !s.starts_with("ERR ") {
            return Err(crate::utils::invalid_input("not an error"));
        }
        for kind in Error::iter() {
            if s == kind.to_string() {
                return Ok(kind);
            }
        }
        Ok(Error::UnknownError)
    }

    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::ConnectionFailed |
            Self::SendFailed |
            Self::ConnectionClosed |
            Self::UnexpectedServerResponse |
            Self::ServerTimeOut,
        )
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::messages::utils::write_vec(f, vec![
            "ERR".to_string(),
            self.code().to_string(),
            self.message().to_string(),
        ])
    }
}
