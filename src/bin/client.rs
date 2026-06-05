use tap::messages::MessageParse;

async fn run_raw() -> Result<(), std::io::Error> {
    let mut player: Option<tap::game::Player> = None;
    let state = tap::cli::State::shared();
    let mut terminal = tap::cli::Terminal::new()?;
    loop {
        tap::cli::raw::ui(
            &mut terminal,
            &*state.lock().await,
            &player,
        );
        let mut state_a = state.clone();
        let state_b = state.clone();
        tokio::select! {
            Some(event) = terminal.read(&mut state_a) => {
                let mut state = state_a.lock().await;
                match event {
                    tap::cli::TerminalEvent::Interrupted => break,
                    tap::cli::TerminalEvent::Validate if !state.waiting => {
                        match &mut player {
                            Some(player) => {
                                let message = state.input.clone();
                                state.messages.push(message);
                                match tap::messages::Message::from_string(&state.input) {
                                    Ok(message) => {
                                        state.waiting = true;
                                        player.client.write(&format!("{message}\n")).await;
                                    },
                                    Err(_) => {
                                        state.messages.push("Error: invalid command".to_string());
                                    },
                                }
                                state.input.clear();
                            }
                            _ => (),
                        }
                    },
                    _ => (),
                }
            }
            _ = async {
                match &mut player {
                    Some(player) => {
                        match player.client.read().await {
                            Ok(message) => {
                                let mut state = state_b.lock().await;
                                match message {
                                    Some(message) => {
                                        state.messages.push(message.to_string());
                                        if matches!(message, tap::messages::Message::Response(_) | tap::messages::Message::Error(_)) {
                                            state.waiting = false;
                                        }
                                    },
                                    None => (),
                                }
                            }
                            _ => (),
                        };
                    }
                    _ => (),
                }
            } => {}
        }
    }
    terminal.close()
}

#[tokio::main]
async fn main() {
    match run_raw().await {
        Err(e) => eprintln!("Error: {e}"),
        _ => (),
    }
}
