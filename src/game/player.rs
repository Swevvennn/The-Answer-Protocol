#[derive(
    Default,
    serde::Deserialize,
    serde::Serialize
)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub username: String,
    pub group: String,
    pub room: String,
    pub items: Vec<String>,

    #[serde(skip)]
    pub writer: Option<std::sync::Arc<crate::network::Writer>>,
}

impl Player {
    pub fn new(username: String, room: String, writer: std::sync::Arc<crate::network::Writer>) -> Self {
        Self {
            username,
            group: String::new(),
            room,
            items: Vec::new(),
            writer: Some(writer),
        }
    }
}
