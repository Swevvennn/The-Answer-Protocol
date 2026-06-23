#[derive(Default)]
pub struct Knowledge {
    pub addr: String,
    pub proto: String,
    pub players: usize,
    pub describes: std::collections::HashSet<String>,
    pub items: std::collections::HashMap<String, crate::game::Item>,
    pub npcs: std::collections::HashMap<String, crate::game::Npc>,
    pub quests: std::collections::HashMap<String, crate::game::Quest>,
    pub room: crate::game::RoomState,
    pub player: crate::game::Player,
}

impl Knowledge {
    pub fn change_room(&mut self, room: crate::game::RoomState) {
        for item in &room.items {
            if !self.items.contains_key(item) {
                self.describes.insert(item.clone());
            }
        }
        for npc in &room.npcs {
            if !self.npcs.contains_key(npc) {
                self.describes.insert(npc.clone());
            }
        }
        self.room = room;
    }

    pub fn update(&mut self, data: crate::game::WorldData) -> Result<(), std::io::Error> {
        match data {
            crate::game::WorldData::Item(item) => {
                self.items.insert(item.id.clone(), item);
            },
            crate::game::WorldData::Npc(npc) => {
                self.npcs.insert(npc.id.clone(), npc);
            },
            crate::game::WorldData::Quest(quest) => {
                self.quests.insert(quest.id.clone(), quest);
            },
            _ => return Err(std::io::Error::other("invalid data kind")),
        };
        Ok(())
    }

    pub fn need(&mut self) -> Option<String> {
        if let Some(v) = self.describes.iter().next().cloned() {
            self.describes.take(&v)
        } else {
            None
        }
    }
}
