#[derive(Default)]
pub struct Player {
    pub username: String,
    pub group: String,
    pub room: String,
    pub items: Vec<String>,
    pub quests: Vec<String>,
    pub completed_quests: std::collections::HashMap<String, usize>,
    pub writer: Option<std::sync::Arc<crate::network::Writer>>,
}

impl Player {
    pub fn new(username: String, writer: std::sync::Arc<crate::network::Writer>) -> Self {
        Self {
            username,
            group: String::new(),
            room: String::new(),
            items: Vec::new(),
            quests: Vec::new(),
            completed_quests: std::collections::HashMap::new(),
            writer: Some(writer),
        }
    }

    pub fn count(game: &crate::game::GameState) -> crate::messages::Message {
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "players".to_string(),
                    value: game.players.len().to_string(),
                },
            ]),
        })
    }

    pub async fn connect(game: &mut crate::game::GameState, client: &mut crate::network::Client, username: &String) -> crate::messages::Message {
        if game.players.contains_key(username) {
            crate::messages::Message::Error(crate::messages::Error::NameInUse)
        } else if username.is_empty() {
            crate::messages::Message::Error(crate::messages::Error::InvalidName)
        } else {
            if let Some(writer) = &client.writer {
                client.state = crate::network::ClientState::Authenticated;
                game.players.insert(
                    username.clone(),
                    crate::game::Player::new(
                        username.clone(),
                        writer.clone(),
                    ),
                );
                crate::cli::Logger::player_count(game).await;
            }
            crate::game::RoomState::enter(game, username, &"room.start".to_string()).await;
            crate::messages::Message::Response(crate::messages::Response {
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String("connected".to_string()),
                ]),
            })
        }
    }

    pub async fn chat(game: &crate::game::GameState, player: &String, scope: &crate::messages::EventScope, message: &String) -> crate::messages::Message {
        let player = &game.players[player];
        crate::cli::Logger::event(
            match scope {
                crate::messages::EventScope::Group => &player.group,
                crate::messages::EventScope::Room => &player.room,
                _ => "",
            },
            game,
            &crate::messages::Event {
                scope: scope.clone(),
                kind: crate::messages::EventKind::Chat,
                payload: crate::messages::Payload::new(&[
                    crate::messages::PayloadKind::String(player.username.clone()),
                    crate::messages::PayloadKind::String(message.clone()),
                ]),
            },
            |to| match scope {
                crate::messages::EventScope::Group => to.group == player.group,
                crate::messages::EventScope::Room => to.room == player.room,
                _ => true,
            },
        ).await;
        crate::messages::Message::Response(crate::messages::Response::default())
    }

    pub fn inventory(game: &crate::game::GameState, player: &String) -> crate::messages::Message {
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::new_json(&game.players[player].items),
            ]),
        })
    }

    pub fn look(game: &crate::game::GameState, player: &String) -> crate::messages::Message {
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::new_json(&game.rooms[&game.players[player].room]),
            ]),
        })
    }

    pub async fn move_to(game: &mut crate::game::GameState, player: &String, direction: &crate::game::Direction) -> crate::messages::Message {
        let room: String;
        if let Some(s) = game.rooms[&game.players[player].room].room.exits.get(&direction) {
            room = s.clone();
        } else {
            return crate::messages::Message::Error(crate::messages::Error::NoExit);
        }
        crate::game::RoomState::leave(game, player).await;
        crate::game::RoomState::enter(game, player, &room).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "room".to_string(),
                    value: room,
                },
            ]),
        })
    }

    pub fn take(game: &mut crate::game::GameState, player: &String, item: &String) -> crate::messages::Message {
        if let Some(player) = game.players.get_mut(player) && let Some(room) = game.rooms.get_mut(&player.room) {
            if let Some(i) = room.items.iter().position(|i| *i == *item) {
                player.items.push(room.items.remove(i));
                crate::messages::Message::Response(crate::messages::Response {
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::KeyValue {
                            key: "taken".to_string(),
                            value: item.clone(),
                        },
                    ]),
                })
            } else {
                crate::messages::Message::Error(crate::messages::Error::ItemNotFound)
            }
        } else {
            crate::messages::Message::Error(crate::messages::Error::ServerError)
        }
    }

    pub fn drop(game: &mut crate::game::GameState, player: &String, item: &String) -> crate::messages::Message {
        if let Some(player) = game.players.get_mut(player) && let Some(room) = game.rooms.get_mut(&player.room) {
            if let Some(i) = player.items.iter().position(|i| *i == *item) {
                room.items.push(player.items.remove(i));
                crate::messages::Message::Response(crate::messages::Response {
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::KeyValue {
                            key: "dropped".to_string(),
                            value: item.clone(),
                        },
                    ]),
                })
            } else {
                crate::messages::Message::Error(crate::messages::Error::ItemNotInInventory)
            }
        } else {
            crate::messages::Message::Error(crate::messages::Error::ServerError)
        }
    }
}
