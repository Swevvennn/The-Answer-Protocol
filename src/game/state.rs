#[derive(Default)]
pub struct GameState {
    pub players: std::collections::HashMap<String, crate::game::Player>,
    pub groups: std::collections::HashMap<String, crate::game::Group>,
    pub start: String,
    pub rooms: std::collections::HashMap<String, crate::game::RoomState>,
    pub items: std::collections::HashMap<String, crate::game::Item>,
}

impl GameState {
    pub fn new(world: crate::game::World) -> Self {
        let mut rooms = std::collections::HashMap::new();
        for room in world.rooms {
            rooms.insert(
                room.id.clone(),
                crate::game::RoomState::new(room),
            );
        }
        let mut items = std::collections::HashMap::new();
        for item in world.items {
            items.insert(
                item.id.clone(),
                item,
            );
        }
        for spawn in world.spawns {
            if let Some(room) = rooms.get_mut(&spawn.room) {
                if items.contains_key(&spawn.id) {
                    room.items.push(spawn.id.clone());
                }
            }
        }
        Self {
            players: std::collections::HashMap::new(),
            groups: std::collections::HashMap::new(),
            start: world.start,
            rooms,
            items,
        }
    }
}
