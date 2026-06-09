enum Stage {
    EnteringAddress,
    WaitingConnection,
    WaitingGreeting,
    EnteringUsername,
    WaitingAuth,
    EnteringCommand,
    WaitingResponse,
}

enum OtherAction {
    Connection(Result<(), std::io::Error>),
    Read(Result<Option<tap::messages::Message>, std::io::Error>),
}

type Action = tap::cli::Action<OtherAction>;

struct Cli {
    stage: Stage,
    waiter: tap::utils::Waiter,
    input: tap::cli::Input,
    messages: tap::cli::Messages,
    player: tap::game::Player,
}

impl tap::cli::Wrapper for Cli {
    type OtherAction = OtherAction;

    fn new() -> Self {
        Self {
            stage: Stage::EnteringAddress,
            waiter: tap::utils::Waiter::default(),
            input: tap::cli::Input::default(),
            messages: tap::cli::Messages::default(),
            player: tap::game::Player::default(),
        }
    }

    async fn draw(&self, terminal: &mut tap::cli::Terminal) {
        terminal.update(|frame| {
            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Length(4),
                    ratatui::layout::Constraint::Min(1),
                    ratatui::layout::Constraint::Length(3),
                ])
                .split(frame.area());
            frame.render_widget(
                ratatui::widgets::Paragraph::new(
                    format!(
                        "Server: {} ({})\nUsername: {}",
                        if self.player.client.addr.is_empty() { "?" } else { &self.player.client.addr },
                        self.player.client.state,
                        if self.player.username.is_empty() { "?" } else { &self.player.username },
                    )
                )
                    .block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                    ),
                chunks[0],
            );
            frame.render_widget(
                ratatui::widgets::Paragraph::new(self.messages.to_string())
                    .block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                    )
                    .scroll((self.messages.messages.len().saturating_sub(chunks[1].height.saturating_sub(2) as usize) as u16, 0)),
                chunks[1],
            );
            frame.render_widget(
                ratatui::widgets::Paragraph::new(format!("> {}", self.input))
                    .block(
                        ratatui::widgets::Block::default()
                            .title(
                                match self.player.client.state {
                                    tap::network::ClientState::Connected => "Enter a username",
                                    tap::network::ClientState::Authenticated => "Enter a command",
                                    _ => "Enter the server address (<IPv4>:<port>)",
                                }
                            )
                            .borders(ratatui::widgets::Borders::ALL),
                    ), 
                chunks[2],
            );
        });
    }

    async fn select(&mut self, terminal: &mut tap::cli::Terminal) -> Option<Action> {
        tokio::select! {
            _ = self.waiter.wait() => Some(Action::Awake),
            event = terminal.read(&mut self.input) => {
                match event {
                    Some(event) => match event {
                        tap::cli::TerminalEvent::Interrupted => Some(Action::Interrupt),
                        tap::cli::TerminalEvent::Validate => Some(Action::Validate),
                        _ => None,
                    }
                    _ => None,
                }
            }
            action = async {
                if matches!(self.stage, Stage::WaitingConnection) {
                    Some(Action::Other(OtherAction::Connection(self.player.client.connect().await)))
                } else if matches!(self.player.client.state, tap::network::ClientState::Disconnected | tap::network::ClientState::Terminated) {
                    tap::utils::Waiter::block().await;
                    None
                } else {
                    Some(Action::Other(OtherAction::Read(self.player.client.read().await)))
                }
            } => action,
        }
    }

    async fn process(&mut self, action: Action) -> Result<(), std::io::Error> {
        match action {
            Action::Other(OtherAction::Connection(r)) => match r {
                Ok(_) => {
                    self.messages.log(tap::cli::Message::Head(format!(
                        "Connected to {}",
                        self.player.client.addr,
                    )));
                    self.stage = Stage::WaitingGreeting;
                    self.waiter.begin();
                }
                Err(e) => {
                    self.messages.log(tap::cli::Message::error(e));
                    self.stage = Stage::EnteringAddress;
                    self.waiter.end();
                }
            }
            Action::Other(OtherAction::Read(r)) => match r {
                Ok(message) => {
                    if let Some(message) = message {
                        if match (&self.stage, &message) {
                            (Stage::WaitingGreeting, tap::messages::Message::Response(message)) => {
                                let mut version = String::new();
                                match message.payload.extract(&mut [
                                    tap::messages::PayloadExtractor::String(&mut "hello".to_string()),
                                    tap::messages::PayloadExtractor::KeyValue {
                                        key: &mut "proto".to_string(),
                                        value: &mut version
                                    }
                                ]) {
                                    Ok(_) => {
                                        self.stage = Stage::EnteringUsername;
                                        true
                                    }
                                    Err(_) => false,
                                }
                            }
                            (Stage::WaitingAuth, tap::messages::Message::Error(_)) => {
                                self.stage = Stage::EnteringUsername;
                                true
                            }
                            (Stage::WaitingAuth, tap::messages::Message::Response(message)) => match message.payload.extract(&mut [
                                tap::messages::PayloadExtractor::String(&mut "connected".to_string()),
                            ]) {
                                Ok(_) => {
                                    self.stage = Stage::EnteringCommand;
                                    true
                                }
                                _ => false,
                            }
                            (Stage::WaitingResponse, tap::messages::Message::Response(_) | tap::messages::Message::Error(_)) => {
                                self.stage = Stage::EnteringCommand;
                                true
                            }
                            (Stage::EnteringCommand, tap::messages::Message::Event(_)) => true,
                            (_, _) => false,
                        } {
                            self.messages.log(tap::cli::Message::Network {
                                from: "S".to_string(),
                                to: "C".to_string(),
                                message: message.to_string(),
                            });
                        } else {
                            self.messages.log(tap::cli::Message::Error(format!("unexpected message received from the server: {message}")));
                            self.player.client.close();
                        }
                        self.waiter.end();
                    }
                }
                Err(e) => self.messages.log(tap::cli::Message::error(e)),
            }
            Action::Awake => {
                self.messages.log(tap::cli::Message::Error("the server is not responding".to_string()));
                self.player.client.close();
            }
            Action::Validate => {
                match &self.stage {
                    Stage::EnteringAddress => {
                        self.player.client.addr = self.input.consume();
                        self.messages.log(tap::cli::Message::Info(format!(
                            "attempting to connect to '{}'",
                            self.player.client.addr,
                        )));
                        self.stage = Stage::WaitingConnection;
                        self.waiter.begin();
                    }
                    Stage::EnteringUsername => {
                        self.player.username = self.input.consume();
                        self.messages.log(tap::cli::Message::Info(format!(
                            "try to authenticate with username '{}'",
                            self.player.username,
                        )));
                        match self.player.client.write_message(&tap::messages::Message::Command(tap::messages::Command {
                            kind: tap::messages::CommandKind::Connect,
                            payload: tap::messages::Payload::new(&[
                                tap::messages::PayloadKind::String(self.player.username.clone()),
                            ]),
                        })).await {
                            Ok(_) => {
                                self.stage = Stage::WaitingAuth;
                                self.waiter.begin();
                            },
                            Err(e) => self.messages.log(tap::cli::Message::error(e)),
                        };
                    }
                    Stage::EnteringCommand => {
                        let input = self.input.consume();
                        self.messages.log(tap::cli::Message::Network {
                            from: "C".to_string(),
                            to: "S".to_string(),
                            message: input.clone(),
                        });
                        match tap::messages::Message::from_string(&input) {
                            Ok(message) => match self.player.client.write_message(&message).await {
                                Ok(_) => {
                                    self.stage = Stage::WaitingResponse;
                                    self.waiter.begin();
                                },
                                Err(e) => self.messages.log(tap::cli::Message::error(e)),
                            },
                            Err(_) => self.messages.log(tap::cli::Message::Error("invalid command".to_string())),
                        }
                    }
                    _ => (),
                };
            }
            _ => (),
        };
        Ok(())
    }

    async fn update(&mut self) {
        if matches!(self.player.client.state, tap::network::ClientState::Terminated) {
            self.messages.log(tap::cli::Message::Head(format!(
                "Connection to {} closed",
                self.player.client.addr,
            )));
            self.player.client.state = tap::network::ClientState::Disconnected;
            self.stage = Stage::EnteringAddress;
            self.waiter.end();
        };
    }
}

#[tokio::main]
async fn main() {
    tap::cli::run::<Cli>().await;
}
