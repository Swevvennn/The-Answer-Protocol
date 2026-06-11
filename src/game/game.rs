#[derive(Default)]
pub struct Game {
    pub players: std::collections::HashMap<String, crate::game::Player>,
    pub groups: std::collections::HashMap<String, Vec<String>>,
    pub start: String,
    pub rooms: std::collections::HashMap<String, crate::game::RoomState>,
}

impl Game {
    pub fn new(world: crate::game::World) -> Self {
        let mut rooms = std::collections::HashMap::new();
        for room in world.rooms {
            rooms.insert(
                room.id.clone(),
                crate::game::RoomState {
                    room,
                    players: Vec::new(),
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

    pub async fn broadcast_event(&mut self, from: &crate::game::Player, event: &crate::messages::Event) {
        let message = crate::messages::Message::Event(event.clone());
        match event.scope {
            crate::messages::EventScope::Global | crate::messages::EventScope::Stats => {
                for (_, player) in &mut self.players {
                    let _ = player.client.write_message(&message).await;
                }
            }
            crate::messages::EventScope::Group => {
                for (_, player) in &mut self.players {
                    if player.group == from.group {
                        let _ = player.client.write_message(&message).await;
                    }
                }
            }
            crate::messages::EventScope::Room => {
                for (_, player) in &mut self.players {
                    if player.room == from.room {
                        let _ = player.client.write_message(&message).await;
                    }
                }
            }
        };
    }
}
