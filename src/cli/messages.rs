pub enum Message {
    Error(String),
    Head(String),
    Info(String),
    Network {
        from: String,
        to: String,
        message: String
    },
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
            Self::Info(v) => write!(f, "Info: {v}"),
            Self::Network { from, to, message } => write!(f, "{from} -> {to}: {message}"),
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
        if let Some(last) = self.messages.last() && (
            !(matches!(last, Message::Head(_)) || matches!(message, Message::Head(_))) &&
            !(matches!(last, Message::Error(_) | Message::Info(_)) && matches!(message, Message::Error(_) | Message::Info(_)))
        ) {
            self.messages.push(Message::Blank);
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
