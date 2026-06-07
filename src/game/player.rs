use crate::network::Client;

pub struct Player {
    pub client: Client,
    pub username: String,
}

pub type SharedPlayer = std::sync::Arc<tokio::sync::Mutex<Player>>;

impl Player {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            username: String::new(),
        }
    }

    pub fn shared(player: Player) -> SharedPlayer {
        std::sync::Arc::new(
            tokio::sync::Mutex::new(
                player
            )
        )
    }
}
