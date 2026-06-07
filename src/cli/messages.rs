pub enum Message {
    Error(String),
    Head(String),
    Incoming(String),
    Info(String),
    Network {
        from: String,
        to: String,
        message: crate::messages::Message
    },
    Outgoing(String),
    Blank,
}

impl Message {
    pub fn error(e: std::io::Error) -> Self {
        Self::Error(e.to_string())
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(v) => write!(f, "Error: {v}"),
            Self::Head(v) => write!(f, "\n===== {v} =====\n"),
            Self::Incoming(v) => write!(f, "S -> C: {v}"),
            Self::Info(v) => write!(f, "Info: {v}"),
            Self::Network { from, to, message } => write!(f, "{from} -> {to}: {message}"),
            Self::Outgoing(v) => write!(f, "C -> S: {v}"),
            Self::Blank => write!(f, ""),
        }
    }
}

pub struct Messages {
    pub messages: Vec<Message>,
}

impl Messages {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn log(&mut self, message: Message) {
        if let Some(last) = self.messages.last() && (!matches!(last, Message::Head(_)) || !matches!(last, Message::Head(_))) {
            let last_is_local = matches!(last, Message::Error(_) | Message::Info(_));
            let new_is_local = matches!(message, Message::Error(_) | Message::Info(_));
            if last_is_local && !new_is_local || !last_is_local && new_is_local {
                self.messages.push(Message::Blank);
            }
        }
        self.messages.push(message);
        while self.messages.len() > 50 {
            self.messages.remove(0);
        }
    }
}

impl std::fmt::Display for Messages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let messages: Vec<String> = self.messages
            .iter()
            .map(Message::to_string)
            .collect();
        write!(f, "{}", messages.join("\n"))
    }
}
