#[derive(Default)]
pub struct Player {
    pub client: crate::network::Client,
    pub username: String,
    pub group: String,
    pub room: String,
}

impl Player {
    pub fn new(client: crate::network::Client) -> Self {
        Self {
            client,
            username: String::new(),
            group: String::new(),
            room: String::new(),
        }
    }

    pub fn process(&mut self, game: &crate::game::Game, command: &crate::messages::Command) -> crate::messages::Message {
        match command.kind {
            crate::messages::CommandKind::Connect => {
                let mut username = String::new();
                if let Err(_) = command.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut username),
                ]) {
                    crate::messages::Message::Error(crate::messages::Error::InvalidArguments)
                } else if game.players.contains_key(&username) {
                    crate::messages::Message::Error(crate::messages::Error::NameInUse)
                } else {
                    // TODO
                    self.username = username;
                    self.client.state = crate::network::ClientState::Authenticated;
                    crate::messages::Message::Response(crate::messages::Response {
                        payload: crate::messages::Payload::new(&[
                            crate::messages::PayloadKind::String("connected".to_string()),
                        ]),
                    })
                }
            }
            crate::messages::CommandKind::Quit => {
                if command.payload.is_empty() {
                    // TODO
                    crate::messages::Message::Response(crate::messages::Response {
                        payload: crate::messages::Payload::new(&[
                            crate::messages::PayloadKind::String("bye".to_string()),
                        ]),
                    })
                } else {
                    crate::messages::Message::Error(crate::messages::Error::InvalidArguments)
                }
            }
            crate::messages::CommandKind::Look => {
                if command.payload.is_empty() {
                    crate::messages::Message::Response(crate::messages::Response {
                        payload: crate::messages::Payload::new(&[
                            crate::messages::PayloadKind::new_json(&game.rooms[&self.room]),
                        ]),
                    })
                } else {
                    crate::messages::Message::Error(crate::messages::Error::InvalidArguments)
                }
            }
            crate::messages::CommandKind::Move => {
                let mut direction = String::new();
                if let Err(_) = command.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut direction),
                ]) {
                    crate::messages::Message::Error(crate::messages::Error::InvalidArguments)
                } else {
                    match crate::game::Direction::from_str(&direction) {
                        Ok(direction) => {
                            let room = &game.rooms[&self.room].room;
                            if room.exits.contains_key(&direction) {
                                self.room = room.exits[&direction].clone();
                                crate::messages::Message::Response(crate::messages::Response {
                                    payload: crate::messages::Payload::new(&[
                                        crate::messages::PayloadKind::KeyValue {
                                            key: "room".to_string(),
                                            value: self.room.clone(),
                                        },
                                    ]),
                                })
                            } else {
                                crate::messages::Message::Error(crate::messages::Error::NoExit)
                            }
                        }
                        Err(_) => return crate::messages::Message::Error(crate::messages::Error::InvalidArguments),
                    }
                }
            }
            crate::messages::CommandKind::Who => {
                if command.payload.is_empty() {
                    crate::messages::Message::Response(crate::messages::Response {
                        payload: crate::messages::Payload::new(&[
                            crate::messages::PayloadKind::KeyValue {
                                key: "players".to_string(),
                                value: game.players.len().to_string(),
                            },
                        ]),
                    })
                } else {
                    crate::messages::Message::Error(crate::messages::Error::InvalidArguments)
                }
            }
            _ => crate::messages::Message::Response(crate::messages::Response::default()),
        }
    }
}
