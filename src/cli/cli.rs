use crate::cli::{State, Terminal, Waiter};
use crate::game::Player;

pub enum CliStage {
    EnteringAddress,
    WaitingConnection,
    WaitingGreeting,
    EnteringUsername,
    WaitingAuth,
    EnteringCommand,
    WaitingResponse,
}

pub struct Cli {
    pub stage: CliStage,
    pub waiter: Waiter,
    pub state: State,
    pub player: Player,
    pub terminal: Terminal,
}

impl Cli {
    pub fn new() -> Result<Self, std::io::Error> {
        Ok(Self {
            stage: CliStage::EnteringAddress,
            waiter: Waiter::new(),
            state: State::new(),
            player: Player::new(),
            terminal: match Terminal::new() {
                Err(e) => return Err(std::io::Error::new(
                    e.kind(),
                    format!("failed to create terminal ui: {e}"),
                )),
                Ok(v) => v,
            },
        })
    }

    pub fn update<F: FnOnce(&State, &Player, &mut ratatui::Frame)>(&mut self, ui: F) {
        match self.terminal.tui.draw(|frame| {
            ui(
                &self.state,
                &self.player,
                frame,
            );
        }) {
            _ => (),
        }
    }
}
