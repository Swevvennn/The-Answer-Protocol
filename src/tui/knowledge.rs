#[derive(Default)]
pub struct Knowledge {
    pub addr: String,
    pub proto: String,
    pub players: usize,
    pub describes: std::collections::HashSet<String>,
    pub items: std::collections::HashMap<String, crate::game::Item>,
    pub npcs: std::collections::HashMap<String, crate::game::Npc>,
    pub quests: std::collections::HashMap<String, crate::game::Quest>,
    pub pos: (i32, i32),
    pub positions: std::collections::HashMap<String, (i32, i32)>,
    pub rooms: std::collections::HashMap<(i32, i32), (String, String)>,
    pub connections: std::collections::HashSet<((i32, i32), (i32, i32))>,
    pub room: crate::game::RoomState,
    pub group: crate::game::Group,
    pub player: crate::game::Player,
}

impl Knowledge {
    pub fn change_room(&mut self, room: crate::game::RoomState) {
        if !self.positions.contains_key(&room.room.id) {
            let mut pos = (0, 0);
            for (direction, id) in &room.room.exits {
                if let Some(other) = self.positions.get(id) {
                    match direction {
                        crate::game::Direction::East => pos = (other.0 - 1, other.1),
                        crate::game::Direction::North => pos = (other.0, other.1 + 1),
                        crate::game::Direction::South => pos = (other.0, other.1 - 1),
                        crate::game::Direction::West => pos = (other.0 + 1, other.1),
                    }
                    break;
                }
            }
            self.positions.insert(room.room.id.clone(), pos);
            self.rooms.insert(pos, (room.room.id.clone(), room.room.name.clone()));
            for direction in room.room.exits.keys() {
                match direction {
                    crate::game::Direction::East => self.connections.insert((
                        pos,
                        (pos.0 + 1, pos.1),
                    )),
                    crate::game::Direction::North => self.connections.insert((
                        (pos.0, pos.1 - 1),
                        pos,
                    )),
                    crate::game::Direction::South => self.connections.insert((
                        pos,
                        (pos.0, pos.1 + 1),
                    )),
                    crate::game::Direction::West => self.connections.insert((
                        (pos.0 - 1, pos.1),
                        pos,
                    )),
                };
            }
        }
        self.room = room;
    }

    pub fn update(&mut self, data: crate::game::WorldData) -> Result<(), std::io::Error> {
        match data {
            crate::game::WorldData::Group(group) => {
                self.describes.remove(&group.name);
                self.group = group;
            }
            crate::game::WorldData::Item(item) => {
                self.describes.remove(&item.id);
                self.items.insert(item.id.clone(), item);
            },
            crate::game::WorldData::Npc(npc) => {
                self.describes.remove(&npc.id);
                self.npcs.insert(npc.id.clone(), npc);
            },
            crate::game::WorldData::Quest(quest) => {
                self.describes.remove(&quest.id);
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
