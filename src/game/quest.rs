#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub enum QuestKind {
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requires: Vec<QuestKind>,
    pub reward: Vec<String>,
}
