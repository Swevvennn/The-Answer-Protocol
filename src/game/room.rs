#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub description: String,
    pub exits: std::collections::HashMap<crate::game::Direction, String>,
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct RoomState {
    pub room: Room,
    pub players: Vec<String>,
    // pub items: Vec<String>,
    // pub npcs: Vec<String>,
}
