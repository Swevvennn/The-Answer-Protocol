use crate::messages::{Command, Error, Event, Response};

pub enum Message {
    Command(Command),
    Error(Error),
    Event(Event),
    Response(Response),
}
