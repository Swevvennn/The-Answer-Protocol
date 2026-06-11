#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub username: String,
    pub group: String,
    pub room: String,

    #[serde(skip)]
    pub client: crate::network::Client,

    #[serde(skip)]
    pub command: Option<crate::messages::Command>,
}

impl Player {
    pub fn new(client: crate::network::Client) -> Self {
        Self {
            username: String::new(),
            group: String::new(),
            room: String::new(),
            client,
            command: None,
        }
    }

    pub fn is_waiting_response(&self) -> bool {
        self.command.is_some()
    }

    pub async fn run(&mut self, game: crate::utils::Shared<crate::game::Game>) {
        let _ = self.client.write_message(&crate::messages::Message::Response(crate::messages::Response {
            payload: crate::messages::Payload::new(&[
                crate::messages::PayloadKind::String("hello".to_string()),
                crate::messages::PayloadKind::KeyValue {
                    key: "proto".to_string(),
                    value: "1".to_string(),
                }
            ])
        })).await;
        loop {
            match self.client.read().await {
                Ok(Some(message)) => {
                    if let crate::messages::Message::Command(command) = message {
                        if command.kind.requires_auth() && !matches!(self.client.state, crate::network::ClientState::Authenticated) {
                            let _ = self.client.write_message(&crate::messages::Message::Error(crate::messages::Error::NotAuthenticated)).await;
                        } else if matches!(command.kind, crate::messages::CommandKind::Connect) && matches!(self.client.state, crate::network::ClientState::Authenticated) {
                            let _ = self.client.write_message(&crate::messages::Message::Error(crate::messages::Error::AlreadyAuthenticated)).await;
                        } else {
                            let game = game.lock().await;
                            let message = self.process_command(&game, &command).await;
                            let _ = self.client.write_message(&message).await;
                            if let crate::messages::Message::Response(response) = message && (
                                self.client.is_open() &&
                                response.payload.args.len() == 1 &&
                                if let crate::messages::PayloadKind::String(s) = &response.payload.args[0] {
                                    s == "bye"
                                } else {
                                    false
                                }
                            ) {
                                self.client.close();
                            }
                        }
                    } else {
                        let _ = self.client.write_message(&crate::messages::Message::Error(crate::messages::Error::NotACommand)).await;
                    }
                }
                Ok(None) => (),
                Err(_) => {
                    if self.client.is_open() {
                        let _ = self.client.write_message(&crate::messages::Message::Error(crate::messages::Error::NotACommand)).await;
                    } else {
                        break;
                    }
                }
            };
            if !self.client.is_open() {
                break;
            }
        }
    }

    pub async fn send_command(&mut self, command: crate::messages::Command) -> Option<crate::messages::Error> {
        self.command = Some(command.clone());
        match self.client.write_message(&crate::messages::Message::Command(command)).await {
            Ok(_) => None,
            Err(_) => Some(crate::messages::Error::SendFailed),
        }
    }

    pub async fn process_command(&mut self, game: &crate::game::Game, command: &crate::messages::Command) -> crate::messages::Message {
        match command.kind {
            crate::messages::CommandKind::Connect => {
                self.username.clear();
                if let Err(_) = command.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut self.username),
                ]) {
                    crate::messages::Message::Error(crate::messages::Error::InvalidArguments)
                } else if game.players.contains_key(&self.username) {
                    crate::messages::Message::Error(crate::messages::Error::NameInUse)
                } else if self.username.is_empty() {
                    crate::messages::Message::Error(crate::messages::Error::InvalidName)
                } else {
                    self.client.state = crate::network::ClientState::Authenticated;
                    self.room = game.start.clone();
                    crate::messages::Message::Response(crate::messages::Response {
                        payload: crate::messages::Payload::new(&[
                            crate::messages::PayloadKind::String("connected".to_string()),
                        ]),
                    })
                }
            }
            crate::messages::CommandKind::Quit => {
                if command.payload.is_empty() {
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

    pub fn process_response(&mut self, response: &crate::messages::Response) -> Option<crate::messages::Error> {
        match &self.command {
            Some(command) => match command.kind {
                crate::messages::CommandKind::Connect => match response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "connected".to_string()),
                ]) {
                    Ok(_) => {
                        self.client.state = crate::network::ClientState::Authenticated;
                        self.username.clear();
                        if let Err(_) = command.payload.extract(&mut [
                            crate::messages::PayloadExtractor::String(&mut self.username),
                        ]) {
                            return Some(crate::messages::Error::UnexpectedServerResponse);
                        }
                    },
                    Err(_) => return Some(crate::messages::Error::UnexpectedServerResponse),
                }
                _ => (),
            }
            None => {
                self.client.proto.clear();
                match response.payload.extract(&mut [
                    crate::messages::PayloadExtractor::String(&mut "hello".to_string()),
                    crate::messages::PayloadExtractor::KeyValue {
                        key: &mut "proto".to_string(),
                        value: &mut self.client.proto,
                    }
                ]) {
                    Ok(_) => (),
                    Err(_) => return Some(crate::messages::Error::UnexpectedServerResponse),
                }
            },
        };
        self.command = None;
        None
    }
}
