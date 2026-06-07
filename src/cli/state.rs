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
}
