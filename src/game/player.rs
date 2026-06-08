use crate::network::Client;

pub struct Player {
    pub client: Client,
    pub username: String,
}

impl Player {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            username: String::new(),
        }
    }
}
