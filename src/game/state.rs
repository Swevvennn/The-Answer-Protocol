fn default_count() -> usize {
    1
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
enum SpawnKind {
    Item {
        room: String,
        item: String,

        #[serde(default = "default_count")]
        count: usize,
    },
    NPC {
        room: String,
        npc: String,
    },
}

#[derive(
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
struct World {
    pub rooms: Vec<crate::game::Room>,
    pub items: Vec<crate::game::Item>,
    pub npcs: Vec<crate::game::NPC>,
    pub quests: Vec<crate::game::Quest>,
    pub spawns: Vec<SpawnKind>,
}

#[derive(Default)]
pub struct GameState {
    pub players: std::collections::HashMap<String, crate::game::Player>,
    pub groups: std::collections::HashMap<String, crate::game::Group>,
    pub rooms: std::collections::HashMap<String, crate::game::RoomState>,
    pub items: std::collections::HashMap<String, crate::game::Item>,
    pub npcs: std::collections::HashMap<String, crate::game::NPC>,
}

impl GameState {
    pub fn new(path: &str) -> Result<Self, std::io::Error> {
        let world: World = serde_json::from_str(
            &std::fs::read_to_string(path)?
        )?;
        if world.rooms.is_empty() {
            return Err(std::io::Error::other("cannot instantiate a world without any room data"));
        }
        let mut game = Self::default();
        for mut room in world.rooms {
            room.id = format!("room.{}", room.id);
            if game.rooms.contains_key(&room.id) {
                return Err(std::io::Error::other(format!(
                    "duplicated room id '{}'",
                    room.id,
                )));
            }
            for (_, id) in room.exits.iter_mut() {
                *id = format!("room.{id}");
            }
            game.rooms.insert(
                room.id.clone(),
                crate::game::RoomState::new(room),
            );
        }
        if !game.rooms.contains_key("room.start") {
            return Err(std::io::Error::other(format!("missing room 'start' used as spawn point")));
        }
        let mut positions: std::collections::HashMap<String, (i32, i32)> = std::collections::HashMap::new();
        loop {
            for (id, room) in &game.rooms {
                let (mut x, mut y) = (None, None);
                for (direction, other) in &room.room.exits {
                    if let Some(other) = game.rooms.get(other) {
                        if !other.room.exits.contains_key(&direction.opposite()) || other.room.exits[&direction.opposite()] != room.room.id {
                            return Err(std::io::Error::other(format!(
                                "the path between '{}' and '{}' is not reversible",
                                id,
                                other.room.id,
                            )));
                        }
                    } else {
                        return Err(std::io::Error::other(format!("there is no room identified by '{other}'")));
                    }
                    if let Some(dest) = positions.get(other) {
                        if let Some(x) = x && let Some(y) = y {
                            if match direction {
                                crate::game::Direction::East => x != dest.0 - 1 || y != dest.1,
                                crate::game::Direction::North => x != dest.0 || y != dest.1 + 1,
                                crate::game::Direction::South => x != dest.0 || y != dest.1 - 1,
                                crate::game::Direction::West => x != dest.0 + 1 || y != dest.1,
                            } {
                                return Err(std::io::Error::other("the rooms are not arranged in a geometrically correct way"));
                            }
                        } else {
                            match direction {
                                crate::game::Direction::East => { x = Some(dest.0 - 1); y = Some(dest.1); }
                                crate::game::Direction::North => { x = Some(dest.0); y = Some(dest.1 + 1); }
                                crate::game::Direction::South => { x = Some(dest.0); y = Some(dest.1 - 1); }
                                crate::game::Direction::West => { x = Some(dest.0 + 1); y = Some(dest.1); }
                            }
                        }
                    }
                }
                if let Some(x) = x && let Some(y) = y {
                    positions.insert(id.clone(), (x, y));
                    continue;
                } else if positions.is_empty() {
                    positions.insert(id.clone(), (0, 0));
                    continue;
                }
            }
            break;
        }
        for mut item in world.items {
            item.id = format!("item.{}", item.id);
            if game.items.contains_key(&item.id) {
                return Err(std::io::Error::other(format!(
                    "duplicated item id '{}'",
                    item.id,
                )));
            }
            game.items.insert(
                item.id.clone(),
                item,
            );
        }
        for mut npc in world.npcs {
            npc.id = format!("npc.{}", npc.id);
            if game.npcs.contains_key(&npc.id) {
                return Err(std::io::Error::other(format!(
                    "duplicated NPC id '{}'",
                    npc.id,
                )));
            }
            game.npcs.insert(
                npc.id.clone(),
                npc,
            );
        }
        for spawn in world.spawns {
            match spawn {
                SpawnKind::Item {
                    mut room,
                    mut item,
                    count
                } => {
                    room = format!("room.{room}");
                    item = format!("item.{item}");
                    if !game.items.contains_key(&item) {
                        return Err(std::io::Error::other(format!("there is no item identified by '{item}'")));
                    }
                    if let Some(room) = game.rooms.get_mut(&room) {
                        room.items.extend(std::iter::repeat_n(item, count));
                    } else {
                        return Err(std::io::Error::other(format!("there is no room identified by '{room}'")));
                    }
                }
                SpawnKind::NPC {
                    mut room,
                    mut npc,
                } => {
                    room = format!("room.{room}");
                    npc = format!("npc.{npc}");
                    if !game.npcs.contains_key(&npc) {
                        return Err(std::io::Error::other(format!("there is no npc identified by '{npc}'")));
                    }
                    if let Some(room) = game.rooms.get_mut(&room) {
                        room.npcs.push(npc);
                    } else {
                        return Err(std::io::Error::other(format!("there is no room identified by '{room}'")));
                    }
                }
            }
        }
        Ok(game)
    }
}
