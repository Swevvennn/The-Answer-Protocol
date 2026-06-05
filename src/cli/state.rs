pub struct State {
    pub waiting: bool,
    pub input: String,
    pub messages: Vec<String>,
    pub history: Vec<String>,
}

pub type SharedState = std::sync::Arc<tokio::sync::Mutex<State>>;

impl State {
    pub fn new() -> Self {
        Self {
            waiting: false,
            input: String::new(),
            messages: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn shared() -> SharedState {
        std::sync::Arc::new(
            tokio::sync::Mutex::new(
                State::new()
            )
        )
    }
}
