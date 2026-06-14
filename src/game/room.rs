#[derive(
    Default,
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
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct RoomState {
    pub room: Room,
    pub players: std::collections::HashSet<String>,
    pub items: Vec<String>,
    // pub npcs: Vec<String>,
}

impl RoomState {
    pub fn new(room: Room) -> Self {
        Self {
            room,
            players: std::collections::HashSet::new(),
            items: Vec::new(),
        }
    }
}
