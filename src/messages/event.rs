use strum::IntoEnumIterator;

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
    pub payload: crate::messages::Payload,
}

impl Event {
    pub fn from_str(s: &str) -> Result<Event, std::io::Error> {
        let mut message = s.to_string();
        if crate::messages::utils::parse_begin(&mut message, "EVT") {
            return Err(crate::utils::invalid_input("not an event"));
        }
        let err = Err(crate::utils::invalid_input("invalid event"));
        if matches!(crate::messages::utils::skip_space(&mut message), Ok(false) | Err(_)) {
            return err;
        }
        for scope in EventScope::iter() {
            if crate::messages::utils::parse_begin(&mut message, &scope.to_string()) {
                if matches!(crate::messages::utils::skip_space(&mut message), Ok(false) | Err(_)) {
                    return err;
                }
                for kind in EventKind::iter() {
                    if crate::messages::utils::parse_begin(&mut message, &kind.to_string()) {
                        return Ok(Event {
                            scope,
                            kind,
                            payload: match crate::messages::utils::parse_payload(&mut message) {
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
        crate::messages::utils::write_vec(f, vec![
            "EVT".to_string(),
            self.scope.to_string(),
            self.kind.to_string(),
            self.payload.to_string(),
        ])
    }
}
