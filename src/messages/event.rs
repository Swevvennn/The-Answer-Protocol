use std::fmt;
use std::io::Error;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::messages::MessageParse;
use crate::messages::Payload;
use crate::messages::utils::{invalid_input, parse_begin, parse_payload, skip_space, write_vec};

#[derive(EnumIter)]
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

#[derive(EnumIter)]
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

impl MessageParse for Event {
    fn from_string(mut s: String) -> Result<Event, Error> {
        if parse_begin(&mut s, "EVT") {
            return Err(invalid_input("not an event"));
        }
        let err = Err(invalid_input("invalid event"));
        match skip_space(&mut s) {
            Ok(false) => return err,
            Err(_) => return err,
            _ => (),
        };
        for scope in EventScope::iter() {
            if parse_begin(&mut s, &scope.to_string()) {
                match skip_space(&mut s) {
                    Ok(false) => return err,
                    Err(_) => return err,
                    _ => (),
                };
                for kind in EventKind::iter() {
                    if parse_begin(&mut s, &kind.to_string()) {
                        return Ok(Event {
                            scope,
                            kind,
                            payload: match parse_payload(&mut s) {
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

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_vec(f, vec![
            "EVT".to_string(),
            self.scope.to_string(),
            self.kind.to_string(),
            self.payload.to_string(),
        ])
    }
}
