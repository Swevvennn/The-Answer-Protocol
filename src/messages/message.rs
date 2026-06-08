use crate::messages::{Command, Error, Event, Response};
use crate::messages::utils;

pub enum Message {
    Command(Command),
    Error(Error),
    Event(Event),
    Response(Response),
}

impl Message {
    pub fn from_string(s: &str) -> Result<Message, std::io::Error> {
        if let Ok(v) = Command::from_string(s) {
            return Ok(Message::Command(v));
        }
        if let Ok(v) = Error::from_string(s) {
            return Ok(Message::Error(v));
        }
        if let Ok(v) = Event::from_string(s) {
            return Ok(Message::Event(v));
        }
        if let Ok(v) = Response::from_string(s) {
            return Ok(Message::Response(v));
        }
        Err(utils::invalid_input("invalid message"))
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(v) => write!(f, "{v}"),
            Self::Error(v) => write!(f, "{v}"),
            Self::Event(v) => write!(f, "{v}"),
            Self::Response(v) => write!(f, "{v}"),
        }
    }
}
