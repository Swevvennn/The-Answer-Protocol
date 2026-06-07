use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::cli::{Cli, CliStage, Message, State, TerminalEvent, Waiter};
use crate::game::Player;
use crate::messages;
use crate::network::ClientState;

pub async fn run<F: AsyncFn(&mut Cli), G: AsyncFn(&mut Cli, messages::Message)>(validate: F, receive: G) -> Result<(), std::io::Error> {
    enum Action {
        Connection(Result<(), std::io::Error>),
        Interrupt,
        Read(Result<Option<messages::Message>, std::io::Error>),
        Timeout,
        Validate,
    }
    let mut cli = Cli::new()?;
    loop {
        cli.update(ui);
        match tokio::select! {
            _ = cli.waiter.wait() => Some(Action::Timeout),
            event = cli.terminal.read(&mut cli.state) => {
                match event {
                    Some(event) => match event {
                        TerminalEvent::Interrupted => Some(Action::Interrupt),
                        TerminalEvent::Validate => Some(Action::Validate),
                        _ => None,
                    }
                    _ => None,
                }
            }
            action = async {
                if matches!(cli.stage, CliStage::WaitingConnection) {
                    Some(Action::Connection(cli.player.client.connect().await)) 
                } else if matches!(cli.player.client.state, ClientState::Disconnected | ClientState::Terminated) {
                    Waiter::block().await;
                    None
                } else {
                    Some(Action::Read(cli.player.client.read().await))
                }
            } => action,
        } {
            Some(action) => match action {
                Action::Connection(r) => match r {
                    Ok(_) => {
                        cli.state.messages.push(Message::Head(format!(
                            "Connected to {}",
                            cli.player.client.addr,
                        )));
                        cli.stage = CliStage::WaitingGreeting;
                        cli.waiter.begin();
                    }
                    Err(e) => {
                        cli.state.messages.push(Message::error(e));
                        cli.stage = CliStage::EnteringAddress;
                        cli.waiter.end();
                    }
                }
                Action::Interrupt => break,
                Action::Read(r) => match r {
                    Ok(message) => match message {
                        Some(message) => receive(&mut cli, message).await,
                        None => (),
                    }
                    Err(e) => cli.state.messages.push(Message::error(e)),
                }
                Action::Timeout => {
                    cli.state.messages.push(Message::Error("the server is not responding".to_string()));
                    cli.player.client.close();
                }
                Action::Validate => validate(&mut cli).await,
            }
            None => (),
        };
        if matches!(cli.player.client.state, ClientState::Terminated) {
            cli.state.messages.push(Message::Head(format!(
                "Connection to {} closed",
                cli.player.client.addr,
            )));
            cli.player.client.state = ClientState::Disconnected;
            cli.stage = CliStage::EnteringAddress;
        }
    }
    cli.terminal.close()?;
    Ok(())
}

fn ui(state: &State, player: &Player, frame: &mut ratatui::Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());
    frame.render_widget(
        Paragraph::new(
            format!(
                "Server: {} ({})\nUsername: {}",
                if player.client.addr.is_empty() { "?" } else { &player.client.addr },
                player.client.state,
                if player.username.is_empty() { "?" } else { &player.username },
            )
        )
            .block(
                Block::default()
                    .borders(Borders::ALL)
            ),
        chunks[0],
    );
    let messages: Vec<String> = state.messages
        .iter()
        .map(Message::to_string)
        .collect();
    frame.render_widget(
        Paragraph::new(messages.join("\n"))
            .block(
                Block::default()
                    .borders(Borders::ALL)
            )
            .scroll({
                let height = chunks[1].height.saturating_sub(2) as usize;
                let total_lines = state.messages.len();
                let offset = total_lines.saturating_sub(height);
                (offset as u16, 0)
            }),
        chunks[1],
    );
    frame.render_widget(
        Paragraph::new(format!("> {}", state.input))
            .block(
                Block::default()
                    .title(
                        match player.client.state {
                            ClientState::Connected => "Enter a username",
                            ClientState::Authenticated => "Enter a command",
                            _ => "Enter the server address (<IPv4>:<port>)",
                        }
                    )
                    .borders(Borders::ALL),
            ), 
        chunks[2],
    );
}