#[derive(
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct Spawn {
    pub id: String,
    pub room: String,
}

#[derive(
    Default,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub start: String,
    pub rooms: Vec<crate::game::Room>,
    pub items: Vec<crate::game::Item>,
    pub spawns: Vec<Spawn>,
}

#[derive(Default)]
pub struct GameState {
    pub players: std::collections::HashMap<String, crate::game::Player>,
    pub groups: std::collections::HashMap<String, crate::game::Group>,
    pub start: String,
    pub rooms: std::collections::HashMap<String, crate::game::RoomState>,
    pub items: std::collections::HashMap<String, crate::game::Item>,
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
        for room in world.rooms {
            if !room.id.starts_with("room.") {
                return Err(std::io::Error::other(format!(
                    "invalid room id '{}': rooms id must be in format room.<id>",
                    room.id,
                )));
            }
            if game.rooms.contains_key(&room.id) {
                return Err(std::io::Error::other(format!(
                    "duplicated room id '{}'",
                    room.id,
                )));
            }
            game.rooms.insert(
                room.id.clone(),
                crate::game::RoomState::new(room),
            );
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
        for item in world.items {
            if !item.id.starts_with("item.") {
                return Err(std::io::Error::other(format!(
                    "invalid item id '{}': items id must be in format item.<id>",
                    item.id,
                )));
            }
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
        for spawn in world.spawns {
            if let Some(room) = game.rooms.get_mut(&spawn.room) {
                if game.items.contains_key(&spawn.id) {
                    room.items.push(spawn.id.clone());
                }
                else {
                    return Err(std::io::Error::other(format!(
                        "the id '{}' doesn't refer to any world data",
                        spawn.id,
                    )));
                }
            } else {
                return Err(std::io::Error::other(format!(
                    "there is no room identified by '{}'",
                    spawn.room,
                )));
            }
        }
        Ok(game)
    }
}
