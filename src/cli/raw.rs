use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::cli::State;
use crate::cli::Terminal;
use crate::game::Player;
use crate::network::ClientState;

pub fn ui(terminal: &mut Terminal, state: &State, player: &Option<Player>) {
    terminal.update(|frame| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // header
                Constraint::Min(1),    // messages
                Constraint::Length(3), // footer input
            ])
            .split(frame.area());
        frame.render_widget(
            Paragraph::new(
                match &player {
                    Some(player) => format!(
                        "Server: {} ({})\nUsername: {}",
                        if matches!(player.client.state, ClientState::Disconnected) { "?" } else { &player.client.addr },
                        player.client.state,
                        if player.username.is_empty() { "?" } else { &player.username }
                    ),
                    None => "You're not connected to the MUD server".to_string(),
                }
            )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                ),
            chunks[0],
        );
        frame.render_widget(
            Paragraph::new(state.messages.join("\n"))
                .block(
                    Block::default()
                        .title("Messages")
                        .borders(Borders::ALL)
                )
                .scroll({
                    // scroll automatique en bas
                    let height = chunks[1].height.saturating_sub(2) as usize; // - borders
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
                        .title("Command")
                        .borders(Borders::ALL),
                ), 
            chunks[2],
        );
    });
}