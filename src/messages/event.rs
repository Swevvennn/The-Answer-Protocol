use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt;

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

#[derive(Serialize)]
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
    pub data: Value,
}

impl Event {
    pub fn new<T: Serialize>(scope: EventScope, kind: EventKind, data: T) -> Self {
        Self {
            scope,
            kind,
            data: serde_json::to_value(data).unwrap(),
        }
    }

    pub fn extract<T: DeserializeOwned>(&self) -> T {
        serde_json::from_value(self.data.clone()).unwrap()
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = serde_json::to_string(&self.data)
            .map_err(|_| fmt::Error)?;
        write!(
            f,
            "EVT {} {} {}",
            self.scope,
            self.kind,
            data
        )
    }
}
