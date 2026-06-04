use std::io;

use crate::messages::{Command, Error, Event, Response};
use crate::messages::utils::invalid_input;

pub trait MessageParse: Sized {
    fn from_string(s: String) -> Result<Self, io::Error>;
}

pub enum Message {
    Command(Command),
    Error(Error),
    Event(Event),
    Response(Response),
}

impl Message {
    pub fn from_string(s: &str) -> Result<Message, io::Error> {
        if let Ok(v) = Command::from_string(s.to_string()) {
            return Ok(Message::Command(v));
        }
        if let Ok(v) = Error::from_string(s.to_string()) {
            return Ok(Message::Error(v));
        }
        if let Ok(v) = Event::from_string(s.to_string()) {
            return Ok(Message::Event(v));
        }
        if let Ok(v) = Response::from_string(s.to_string()) {
            return Ok(Message::Response(v));
        }
        Err(invalid_input("invalid message"))
    }
}
