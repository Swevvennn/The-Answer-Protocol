use crate::cli::Message;

pub struct State {
    pub input: String,
    pub messages: Vec<Message>,
    pub history: Vec<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn consume(&mut self) -> String {
        let input = self.input.clone();
        self.input.clear();
        input
    }

    pub fn log(&mut self, message: Message) {
        if let Some(last) = self.messages.last() &&
            !matches!(last, Message::Head(_)) &&
            !matches!(last, Message::Head(_)) && (
            matches!(last, Message::Error(_) | Message::Info(_)) && matches!(message, Message::Incoming(_) | Message::Outgoing(_)) ||
            matches!(last, Message::Incoming(_) | Message::Outgoing(_)) && matches!(message, Message::Error(_) | Message::Info(_)) && !(matches!(last, Message::Outgoing(_)) && matches!(message, Message::Error(_))) ||
            matches!(last, Message::Incoming(_)) && matches!(message, Message::Incoming(_)) ||
            matches!(last, Message::Outgoing(_)) && matches!(message, Message::Outgoing(_)) ||
            matches!(last, Message::Incoming(_)) && matches!(message, Message::Outgoing(_)) ||
            matches!(last, Message::Outgoing(_)) && (if let Message::Incoming(message) = &message && message.starts_with("EVT") { true } else { false })
        ) {
            self.messages.push(Message::Blank);
        }
        self.messages.push(message);
        while self.messages.len() > 50 {
            self.messages.remove(0);
        }
    }
}
