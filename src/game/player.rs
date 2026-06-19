#[derive(Default)]
pub struct Player {
    pub username: String,
    pub group: String,
    pub room: String,
    pub items: Vec<String>,
    pub quests: std::collections::HashMap<String, crate::game::QuestProgress>,
    pub writer: Option<std::sync::Arc<crate::network::Writer>>,
}

impl Player {
    pub fn new(username: String, writer: std::sync::Arc<crate::network::Writer>) -> Self {
        Self {
            username,
            group: String::new(),
            room: String::new(),
            items: Vec::new(),
            quests: std::collections::HashMap::new(),
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

    pub async fn chat(game: &crate::game::GameState, player: &String, scope: &crate::messages::EventScope, message: &str) -> crate::messages::Message {
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
                    crate::messages::PayloadKind::String(message.to_string()),
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

    pub fn quests(game: &crate::game::GameState, player: &String) -> crate::messages::Message {
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::new_json(&game.players[player].quests.values().collect::<Vec<&crate::game::QuestProgress>>()),
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
        if let Some(s) = game.rooms[&game.players[player].room].room.exits.get(direction) {
            room = s.clone();
        } else {
            return crate::messages::Message::Error(crate::messages::Error::NoExit);
        }
        crate::game::RoomState::leave(game, player).await;
        crate::game::RoomState::enter(game, player, &room).await;
        Self::update_quests(game, player, "room", &game.players[player].room.clone()).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "room".to_string(),
                    value: room,
                },
            ]),
        })
    }

    pub async fn take(game: &mut crate::game::GameState, player: &String, item: &String) -> crate::messages::Message {
        if let Some(player) = game.players.get_mut(player) && let Some(room) = game.rooms.get_mut(&player.room) {
            if let Some(i) = room.items.iter().position(|i| *i == *item) {
                player.items.push(room.items.remove(i));
            } else {
                return crate::messages::Message::Error(crate::messages::Error::ItemNotFound)
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::ServerError)
        }
        Self::update_quests(game, player, "item", item).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "taken".to_string(),
                    value: item.clone(),
                },
            ]),
        })
    }

    pub fn abandon_quest(game: &mut crate::game::GameState, player: &String, quest: &String) -> crate::messages::Message {
        if let Some(player) = game.players.get_mut(player) {
            if let Some(quest) = player.quests.get_mut(quest) {
                if matches!(quest.status, crate::game::QuestStatus::Active) {
                    quest.status = crate::game::QuestStatus::Abandoned;
                    crate::messages::Message::Response(crate::messages::Response::default())
                } else {
                    crate::messages::Message::Error(crate::messages::Error::QuestNotActive)
                }
            } else {
                crate::messages::Message::Error(crate::messages::Error::QuestNotFound)
            }
        } else {
            crate::messages::Message::Error(crate::messages::Error::ServerError)
        }
    }

    pub async fn drop(game: &mut crate::game::GameState, player: &String, item: &String) -> crate::messages::Message {
        if let Some(player) = game.players.get_mut(player) && let Some(room) = game.rooms.get_mut(&player.room) {
            if let Some(i) = player.items.iter().position(|i| *i == *item) {
                room.items.push(player.items.remove(i));
            } else {
                return crate::messages::Message::Error(crate::messages::Error::ItemNotInInventory)
            }
        } else {
            return crate::messages::Message::Error(crate::messages::Error::ServerError)
        }
        Self::update_quests(game, player, "item", item).await;
        crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::KeyValue {
                    key: "dropped".to_string(),
                    value: item.clone(),
                },
            ]),
        })
    }

    pub async fn update_quests(game: &mut crate::game::GameState, player: &String, kind: &str, value: &str) {
        let mut completed = Vec::new();
        if let Some(player) = game.players.get_mut(player) {
            for (id, quest) in player.quests.iter_mut() {
                if matches!(quest.status, crate::game::QuestStatus::Active) {
                    let reference = &game.quests[id];
                    match &reference.task {
                        crate::game::QuestKind::Bring {
                            item,
                            count,
                        } if kind.is_empty() || kind == "item" && item == value => quest.progress = std::cmp::min(*count, player.items
                            .iter()
                            .filter(|s| **s == *item)
                            .count() as u32),
                        crate::game::QuestKind::Goto {
                            room
                        } if kind.is_empty() || kind == "room" && room == value => quest.progress = (player.room == *room) as u32,
                        crate::game::QuestKind::Kill {
                            enemy,
                            count,
                        } if kind == "kill" && enemy == value => quest.progress = std::cmp::min(*count, quest.progress + 1),
                        crate::game::QuestKind::Talk {
                            npc
                        } if kind == "talk" && npc == value => quest.progress = 1,
                        _ => (),
                    }
                    if reference.autocomplete && quest.is_complete(&game.quests) {
                        if let crate::game::QuestKind::Bring {
                            item,
                            count,
                        } = &reference.task {
                            for _ in 0..*count {
                                player.items.remove(player.items.iter().position(|i| i == item).unwrap());
                            }
                        }
                        player.items.append(&mut reference.reward.clone());
                        quest.status = crate::game::QuestStatus::Completed;
                        completed.push(id.clone());
                    }
                }
            }
        }
        for quest in completed {
            crate::cli::Logger::event(
                player,
                game,
                &crate::messages::Event {
                    scope: crate::messages::EventScope::Player,
                    kind: crate::messages::EventKind::QuestComplete,
                    payload: crate::messages::Payload::new(&[
                        crate::messages::PayloadKind::String(quest),
                    ]),
                },
                |to| to.username == *player,
            ).await;
        }
    }
}
