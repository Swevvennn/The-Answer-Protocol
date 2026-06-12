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

    #[serde(skip)]
    pub writer: Option<std::sync::Arc<crate::network::Writer>>,
}
