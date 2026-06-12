#[derive(Default)]
pub struct GameState {
    pub players: std::collections::HashMap<String, crate::game::Player>,
    pub groups: std::collections::HashMap<String, crate::game::Group>,
    pub start: String,
    pub rooms: std::collections::HashMap<String, crate::game::RoomState>,
}

impl GameState {
    pub fn new(world: crate::game::World) -> Self {
        let mut rooms = std::collections::HashMap::new();
        for room in world.rooms {
            rooms.insert(
                room.id.clone(),
                crate::game::RoomState {
                    room,
                    players: std::collections::HashSet::new(),
                }
            );
        }
        Self {
            players: std::collections::HashMap::new(),
            groups: std::collections::HashMap::new(),
            start: world.start,
            rooms,
        }
    }
}
