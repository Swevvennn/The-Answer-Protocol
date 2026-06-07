use tap::messages::MessageParse;

#[tokio::main]
async fn main() {
    enum Stage {
        EnteringAddress,
        WaitingConnection,
        WaitingGreeting,
        EnteringUsername,
        WaitingAuth,
        EnteringCommand,
        WaitingResponse,
    }
    enum Action {
        Connection(Result<(), std::io::Error>),
        Interrupt,
        Read(Result<Option<tap::messages::Message>, std::io::Error>),
        Timeout,
        Validate,
    }
    let mut terminal = match tap::cli::Terminal::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: failed to create terminal ui: {e}");
            return;
        }
    };
    let mut stage = Stage::EnteringAddress;
    let mut waiter = tap::utils::Waiter::new();
    let mut input = tap::cli::Input::new();
    let mut messages = tap::cli::Messages::new();
    let mut player = tap::game::Player::new();
    loop {
        let a = 0;
        terminal.update(&a, |_, frame| {
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
                        if player.client.addr.is_empty() { "?" } else { &player.client.addr },
                        player.client.state,
                        if player.username.is_empty() { "?" } else { &player.username },
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
                ratatui::widgets::Paragraph::new(format!("> {input}"))
                    .block(
                        ratatui::widgets::Block::default()
                            .title(
                                match player.client.state {
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
        match match tokio::select! {
            _ = waiter.wait() => Some(Action::Timeout),
            event = terminal.read(&mut input) => {
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
                if matches!(stage, Stage::WaitingConnection) {
                    Some(Action::Connection(player.client.connect().await)) 
                } else if matches!(player.client.state, tap::network::ClientState::Disconnected | tap::network::ClientState::Terminated) {
                    tap::utils::Waiter::block().await;
                    None
                } else {
                    Some(Action::Read(player.client.read().await))
                }
            } => action,
        } {
            Some(action) => match action {
                Action::Connection(r) => match r {
                    Ok(_) => {
                        messages.log(tap::cli::Message::Head(format!(
                            "Connected to {}",
                            player.client.addr,
                        )));
                        stage = Stage::WaitingGreeting;
                        waiter.begin();
                        Ok(())
                    }
                    Err(e) => {
                        messages.log(tap::cli::Message::error(e));
                        stage = Stage::EnteringAddress;
                        waiter.end();
                        Ok(())
                    }
                }
                Action::Interrupt => Err(std::io::Error::other("Interrupted")),
                Action::Read(r) => match r {
                    Ok(message) => match message {
                        Some(message) => {
                            if match (&stage, &message) {
                                (Stage::WaitingGreeting, tap::messages::Message::Response(message)) if message.payload.matches(&[
                                    tap::messages::PayloadPattern::String(Some("hello")),
                                    tap::messages::PayloadPattern::KeyValue(Some("proto")),
                                ]) => {
                                    stage = Stage::EnteringUsername;
                                    true
                                }
                                (Stage::WaitingAuth, message) if matches!(
                                    message,
                                    tap::messages::Message::Response(_) |
                                    tap::messages::Message::Error(_)
                                ) => {
                                    if matches!(message, tap::messages::Message::Error(_)) {
                                        stage = Stage::EnteringUsername;
                                        true
                                    } else if let tap::messages::Message::Response(response) = message && response.payload.matches(&[
                                        tap::messages::PayloadPattern::String(Some("connected")),
                                    ]) {
                                        stage = Stage::EnteringCommand;
                                        true
                                    } else {
                                        false
                                    }
                                }
                                (Stage::WaitingResponse, message) if matches!(
                                    message,
                                    tap::messages::Message::Response(_) |
                                    tap::messages::Message::Error(_)
                                ) => {
                                    stage = Stage::EnteringCommand;
                                    true
                                }
                                (Stage::EnteringCommand, message) if matches!(
                                    message,
                                    tap::messages::Message::Event(_)
                                ) => {
                                    true
                                }
                                (_, _) => false,
                            } {
                                messages.log(tap::cli::Message::Incoming(message.to_string()));
                                waiter.end();
                                return;
                            };
                            messages.log(tap::cli::Message::Error(format!("unexpected message received from the server: {message}")));
                            player.client.close();
                            waiter.end();
                            Ok(())
                        },
                        None => Ok(()),
                    }
                    Err(e) => {
                        messages.log(tap::cli::Message::error(e));
                        Ok(())
                    },
                }
                Action::Timeout => {
                    messages.log(tap::cli::Message::Error("the server is not responding".to_string()));
                    player.client.close();
                    Ok(())
                }
                Action::Validate => {
                    match &stage {
                        Stage::EnteringAddress => {
                            player.client.addr = input.consume();
                            messages.log(tap::cli::Message::Info(format!(
                                "attempting to connect to '{}'",
                                player.client.addr,
                            )));
                            stage = Stage::WaitingConnection;
                            waiter.begin();
                        }
                        Stage::EnteringUsername => {
                            player.username = input.consume();
                            messages.log(tap::cli::Message::Info(format!(
                                "try to authenticate with username '{}'",
                                player.username,
                            )));
                            match player.client.write_message(&tap::messages::Message::Command(tap::messages::Command {
                                kind: tap::messages::CommandKind::Connect,
                                payload: tap::messages::Payload::new(&[
                                    tap::messages::PayloadKind::String(player.username.clone()),
                                ]),
                            })).await {
                                Ok(_) => {
                                    stage = Stage::WaitingAuth;
                                    waiter.begin();
                                },
                                Err(e) => messages.log(tap::cli::Message::error(e)),
                            };
                        }
                        Stage::EnteringCommand => {
                            let input = input.consume();
                            messages.log(tap::cli::Message::Outgoing(input.clone()));
                            match tap::messages::Message::from_string(&input) {
                                Ok(message) => match player.client.write_message(&message).await {
                                    Ok(_) => {
                                        stage = Stage::WaitingResponse;
                                        waiter.begin();
                                    },
                                    Err(e) => messages.log(tap::cli::Message::error(e)),
                                },
                                Err(_) => messages.log(tap::cli::Message::Error("invalid command".to_string())),
                            }
                        }
                        _ => (),
                    };
                    Ok(())
                },
            }
            None => Ok(()),
        } {
            Err(e) => {
                match terminal.close() {
                    Ok(_) => (),
                    Err(e) => eprintln!("Error: failed to close terminal ui: {e}"),
                };
                eprintln!("{e}");
                break;
            }
            Ok(_) => (),
        };
        if matches!(player.client.state, tap::network::ClientState::Terminated) {
            messages.log(tap::cli::Message::Head(format!(
                "Connection to {} closed",
                player.client.addr,
            )));
            player.client.state = tap::network::ClientState::Disconnected;
            stage = Stage::EnteringAddress;
        }
    }
}
