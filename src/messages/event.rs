use strum::IntoEnumIterator;

use crate::messages::MessageParse;
use crate::messages::Payload;
use crate::messages::utils;

#[derive(strum_macros::EnumIter)]
pub enum EventScope {
    Global,
    Group,
    Stats,
    Room,
}

impl std::fmt::Display for EventScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global => write!(f, "GLOBAL"),
            Self::Group => write!(f, "GROUP"),
            Self::Stats => write!(f, "STATS"),
            Self::Room => write!(f, "ROOM"),
        }
    }
}

#[derive(strum_macros::EnumIter)]
pub enum EventKind {
    Chat,
    Invite,
    Join,
    Leave,
    Players,
    PresenceEnter,
    PresenceLeave,
}

impl std::fmt::Display for EventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl MessageParse for Event {
    fn from_string(s: &str) -> Result<Event, std::io::Error> {
        let mut message = s.to_string();
        if utils::parse_begin(&mut message, "EVT") {
            return Err(utils::invalid_input("not an event"));
        }
        let err = Err(utils::invalid_input("invalid event"));
        if matches!(utils::skip_space(&mut message), Ok(false) | Err(_)) {
            return err;
        }
        for scope in EventScope::iter() {
            if utils::parse_begin(&mut message, &scope.to_string()) {
                if matches!(utils::skip_space(&mut message), Ok(false) | Err(_)) {
                    return err;
                }
                for kind in EventKind::iter() {
                    if utils::parse_begin(&mut message, &kind.to_string()) {
                        return Ok(Event {
                            scope,
                            kind,
                            payload: match utils::parse_payload(&mut message) {
                                Ok(v) => v,
                                Err(_) => return err,
                            }
                        });
                    }
                }
                return err;
            }
        }
        err
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        utils::write_vec(f, vec![
            "EVT".to_string(),
            self.scope.to_string(),
            self.kind.to_string(),
            self.payload.to_string(),
        ])
    }
}
