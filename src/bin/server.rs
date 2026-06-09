enum OtherAction {
    Bind(Result<(), std::io::Error>),
    Client(Result<tap::network::Client, std::io::Error>),
}

type Action = tap::cli::Action<OtherAction>;

struct Cli {
    sleeper: tap::utils::Sleeper,
    input: tap::cli::Input,
    messages: tap::utils::Shared<tap::cli::Messages>,
    server: tap::network::Server,
}

impl tap::cli::Wrapper for Cli {
    type OtherAction = OtherAction;

    fn new() -> Self {
        Self {
            sleeper: tap::utils::Sleeper::default(),
            input: tap::cli::Input::default(),
            messages: tap::utils::Shared::new(tap::cli::Messages::default()),
            server: tap::network::Server::default(),
        }
    }

    async fn draw(&self, terminal: &mut tap::cli::Terminal) {
        let messages = self.messages.lock().await;
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
                        "Listening at: {} ({})\nClients number: {}",
                        if self.server.addr.is_empty() { "?" } else { &self.server.addr },
                        self.server.state,
                        "?",
                    )
                )
                    .block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                    ),
                chunks[0],
            );
            frame.render_widget(
                ratatui::widgets::Paragraph::new(messages.to_string())
                    .block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                    )
                    .scroll((messages.messages.len().saturating_sub(chunks[1].height.saturating_sub(2) as usize) as u16, 0)),
                chunks[1],
            );
            frame.render_widget(
                ratatui::widgets::Paragraph::new(format!("> {}", self.input))
                    .block(
                        ratatui::widgets::Block::default()
                            .title(
                                match self.server.state {
                                    tap::network::ServerState::Binded => "Press Ctrl + C to exit",
                                    _ => "Enter a binding address (<IPv4>:<port>)",
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
            _ = self.sleeper.wait() => None,
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
                if matches!(self.server.state, tap::network::ServerState::Binded) {
                    Some(Action::Other(OtherAction::Client(self.server.accept().await)))
                } else if self.server.addr.is_empty() {
                    tap::utils::Waiter::block().await;
                    None
                } else {
                    Some(Action::Other(OtherAction::Bind(self.server.bind().await)))
                }
            } => action,
        }
    }

    async fn process(&mut self, action: Action) -> Result<(), std::io::Error> {
        match action {
            Action::Other(OtherAction::Bind(r)) => match r {
                Ok(()) => {
                    self.messages.lock().await.log(tap::cli::Message::Head(format!(
                        "Server listening on {}",
                        self.server.addr,
                    )));
                }
                Err(e) => {
                    self.messages.lock().await.log(tap::cli::Message::error(e));
                    self.server.addr = String::new();
                }
            }
            Action::Other(OtherAction::Client(r)) => match r {
                Ok(mut client) => {
                    let messages = self.messages.clone();
                    let awaker = self.sleeper.new_awaker();
                    tokio::spawn(async move {
                        messages.lock().await.log(tap::cli::Message::Info(format!(
                            "new client connected {}",
                            client.addr,
                        )));
                        awaker.wake().await;
                        let _ = client.write("OK hello proto=1\n").await;
                        loop {
                            match client.read().await {
                                Ok(None) => (),
                                Ok(Some(v)) => {
                                    messages.lock().await.log(tap::cli::Message::Network {
                                        from: client.addr.clone(),
                                        to: "S".to_string(),
                                        message: v.to_string(),
                                    });
                                    awaker.wake().await;
                                    let _ = client.write("OK connected\n").await;
                                },
                                Err(e) => {
                                    messages.lock().await.log(tap::cli::Message::Error(format!(
                                        "client {} error: {}",
                                        client.addr,
                                        e,
                                    )));
                                    break;
                                }
                            };
                        }
                        messages.lock().await.log(tap::cli::Message::Info(format!(
                            "client {} disconnected",
                            client.addr,
                        )));
                        awaker.wake().await;
                    });
                }
                Err(e) => {
                    self.messages.lock().await.log(tap::cli::Message::error(e));
                    self.server.addr = String::new();
                }
            }
            Action::Validate if !matches!(self.server.state, tap::network::ServerState::Binded) => {
                self.server.addr = self.input.consume();
                self.messages.lock().await.log(tap::cli::Message::Info(format!(
                    "trying to bind on '{}'",
                    self.server.addr,
                )));
            }
            _ => (),
        };
        Ok(())
    }

    async fn update(&mut self) {
        if matches!(self.server.state, tap::network::ServerState::Terminated) {
            self.messages.lock().await.log(tap::cli::Message::Head("Server closed".to_string()));
            self.server.state = tap::network::ServerState::Disconnected;
        }
    }
}

#[tokio::main]
async fn main() {
    tap::cli::run::<Cli>().await;
}
