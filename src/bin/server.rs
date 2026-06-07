#[tokio::main]
async fn main() {
    enum Action {
        Bind(Result<(), std::io::Error>),
        Client(Result<tap::network::Client, std::io::Error>),
        Interrupt,
        Validate,
    }
    let mut terminal = match tap::cli::Terminal::new() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: failed to create terminal ui: {e}");
            return;
        }
    };
    let mut input = tap::cli::Input::new();
    let mut messages = tap::cli::Messages::new();
    let mut server = tap::network::Server::new();
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
                        "Listening at: {} ({})\nClients number: {}",
                        if server.addr.is_empty() { "?" } else { &server.addr },
                        server.state,
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
                ratatui::widgets::Paragraph::new(format!("> {input}"))
                    .block(
                        ratatui::widgets::Block::default()
                            .title(
                                match server.state {
                                    tap::network::ServerState::Binded => "Press Ctrl + C to exit",
                                    _ => "Enter a binding address (<IPv4>:<port>)",
                                }
                            )
                            .borders(ratatui::widgets::Borders::ALL),
                    ), 
                chunks[2],
            );
        });
        match match tokio::select! {
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
                if matches!(server.state, tap::network::ServerState::Binded) {
                    Some(Action::Client(server.accept().await))
                } else if server.addr.is_empty() {
                    tap::utils::Waiter::block().await;
                    None
                } else {
                    Some(Action::Bind(server.bind().await))
                }
            } => action,
        } {
            Some(action) => match action {
                Action::Bind(r) => match r {
                    Ok(()) => {
                        messages.log(tap::cli::Message::Head(format!(
                            "Server listening on {}",
                            server.addr,
                        )));
                        Ok(())
                    }
                    Err(e) => {
                        messages.log(tap::cli::Message::error(e));
                        server.addr = String::new();
                        Ok(())
                    }
                }
                Action::Client(r) => match r {
                    Ok(client) => {
                        Ok(())
                    }
                    Err(e) => {
                        Ok(())
                    }
                }
                Action::Interrupt => Err(std::io::Error::other("Interrupted")),
                Action::Validate => {
                    if !matches!(server.state, tap::network::ServerState::Binded) {
                        server.addr = input.consume();
                        messages.log(tap::cli::Message::Info(format!(
                            "trying to bind on '{}'",
                            server.addr,
                        )));
                    }
                    Ok(())
                }
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
        if matches!(server.state, tap::network::ServerState::Terminated) {
            messages.log(tap::cli::Message::Head("Server closed".to_string()));
            server.state = tap::network::ServerState::Disconnected;
        }
    }
}
