

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
    let messages = tap::utils::Shared::new(tap::cli::Messages::new());
    let (tx, mut rx) = tokio::sync::watch::channel(());
    let tx = tap::utils::Shared::new(tx);
    let mut server = tap::network::Server::new();
    loop {
        {
            let messages = messages.lock().await;
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
        }
        let action = tokio::select! {
            _ = rx.changed() => None,
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
        };
        let mut e: Result<(), std::io::Error> = Ok(());
        match action {
            Some(action) => match action {
                Action::Bind(r) => match r {
                    Ok(()) => {
                        messages.lock().await.log(tap::cli::Message::Head(format!(
                            "Server listening on {}",
                            server.addr,
                        )));
                    }
                    Err(e) => {
                        messages.lock().await.log(tap::cli::Message::error(e));
                        server.addr = String::new();
                    }
                }
                Action::Client(r) => match r {
                    Ok(mut client) => {
                        let messages = messages.clone();
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            messages.lock().await.log(tap::cli::Message::Info(format!(
                                "new client connected {}",
                                client.addr,
                            )));
                            let _ = tx.lock().await.send(());
                            client.write("OK hello proto=1\n").await;
                            loop {
                                match client.read().await {
                                    Ok(None) => (),
                                    Ok(Some(v)) => {
                                        messages.lock().await.log(tap::cli::Message::Network {
                                            from: client.addr.clone(),
                                            to: "S".to_string(),
                                            message: v.to_string(),
                                        });
                                        let _ = tx.lock().await.send(());
                                        client.write("OK connected\n").await;
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
                            let _ = tx.lock().await.send(());
                        });
                    }
                    Err(e) => {
                        messages.lock().await.log(tap::cli::Message::error(e));
                        server.addr = String::new();
                    }
                }
                Action::Interrupt => e = Err(std::io::Error::other("Interrupted")),
                Action::Validate => {
                    if !matches!(server.state, tap::network::ServerState::Binded) {
                        server.addr = input.consume();
                        messages.lock().await.log(tap::cli::Message::Info(format!(
                            "trying to bind on '{}'",
                            server.addr,
                        )));
                    }
                }
            }
            None => (),
        }
        match e {
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
            messages.lock().await.log(tap::cli::Message::Head("Server closed".to_string()));
            server.state = tap::network::ServerState::Disconnected;
        }
    }
}
