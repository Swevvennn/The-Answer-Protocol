#[derive(Default)]
pub struct Game {
    pub players: std::collections::HashMap<String, crate::game::Player>,
    pub groups: std::collections::HashMap<String, Vec<String>>,
    pub rooms: std::collections::HashMap<String, crate::game::RoomState>,
}
