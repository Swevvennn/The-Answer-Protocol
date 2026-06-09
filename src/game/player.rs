#[derive(Default)]
pub struct Player {
    pub client: crate::network::Client,
    pub username: String,
}
