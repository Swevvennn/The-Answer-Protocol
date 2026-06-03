use std::fmt;

use crate::messages::Payload;

pub enum EventScope {
    Global,
    Group,
    Stats,
    Room,
}

impl fmt::Display for EventScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Global => write!(f, "GLOBAL"),
            Self::Group => write!(f, "GROUP"),
            Self::Stats => write!(f, "STATS"),
            Self::Room => write!(f, "ROOM"),
        }
    }
}

pub enum EventKind {
    Chat,
    Invite,
    Join,
    Leave,
    Players,
    PresenceEnter,
    PresenceLeave,
}

impl fmt::Display for EventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Chat => write!(f, "CHAT"),
            Self::Invite => write!(f, "INVITE"),
            Self::Join => write!(f, "JOIN"),
            Self::Leave => write!(f, "LEAVE"),
            Self::Players => write!(f, "PLAYERS"),
            Self::PresenceEnter => write!(f, "PRESENCE ENTER"),
            Self::PresenceLeave => write!(f, "PRESENCE LEAVE"),
        }
    }
}

pub struct Event {
    pub scope: EventScope,
    pub kind: EventKind,
    pub payload: Payload,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EVT {} {} {}",
            self.scope,
            self.kind,
            self.payload
        )
    }
}
