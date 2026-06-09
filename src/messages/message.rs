pub enum Message {
    Command(crate::messages::Command),
    Error(crate::messages::Error),
    Event(crate::messages::Event),
    Response(crate::messages::Response),
}

impl Message {
    pub fn from_string(s: &str) -> Result<Message, std::io::Error> {
        if let Ok(v) = crate::messages::Command::from_string(s) {
            return Ok(Message::Command(v));
        }
        if let Ok(v) = crate::messages::Error::from_string(s) {
            return Ok(Message::Error(v));
        }
        if let Ok(v) = crate::messages::Event::from_string(s) {
            return Ok(Message::Event(v));
        }
        if let Ok(v) = crate::messages::Response::from_string(s) {
            return Ok(Message::Response(v));
        }
        Err(crate::utils::invalid_input("invalid message"))
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
