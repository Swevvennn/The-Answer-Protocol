use crate::network::Client;

#[derive(Default)]
pub struct Player {
    pub client: Client,
    pub username: String,
}
