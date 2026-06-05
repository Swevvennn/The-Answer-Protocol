use crate::network::Client;

pub struct Player {
    pub client: Client,
    pub username: String,
}

pub type SharedPlayer = std::sync::Arc<tokio::sync::Mutex<Player>>;

impl Player {
    pub fn new(client: Client, username: &str) -> Self {
        Self {
            client,
            username: username.to_string(),
        }
    }

    pub fn shared(client: Client, username: &str) -> SharedPlayer {
        std::sync::Arc::new(
            tokio::sync::Mutex::new(
                Player::new(
                    client,
                    username,
                )
            )
        )
    }
}
