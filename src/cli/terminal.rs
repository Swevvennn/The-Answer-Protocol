use futures::StreamExt;

use crate::cli::State;

pub type KeyCode = crossterm::event::KeyCode;
pub type KeyModifiers = crossterm::event::KeyModifiers;

pub enum TerminalEvent {
    Char(char),
    Input,
    Interrupted,
    Other {
        code: KeyCode,
        modifiers: KeyModifiers,
    },
    Validate,
}

pub struct Terminal {
    pub input: String,
    pub tui: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    events: crossterm::event::EventStream,
}

impl Terminal {
    pub fn new() -> Result<Self, std::io::Error> {
        let mut stdout = std::io::stdout();
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        Ok(Self {
            tui: ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))?,
            input: String::new(),
            events: crossterm::event::EventStream::new(),
        })
    }

    pub fn close(&mut self) -> Result<(), std::io::Error> {
        crossterm::execute!(self.tui.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
        self.tui.show_cursor()?;
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    pub async fn read(&mut self, state: &mut State) -> Option<TerminalEvent> {
        match self.events.next().await {
            Some(Ok(event)) => {
                match event {
                    crossterm::event::Event::Key(key) if key.kind == crossterm::event::KeyEventKind::Press => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Some(TerminalEvent::Interrupted),
                            (KeyCode::Char(c), _) => {
                                state.input.push(c);
                                return Some(TerminalEvent::Char(c))
                            }
                            (KeyCode::Backspace, _) => {
                                state.input.pop();
                            }
                            (KeyCode::Enter, _) if !state.input.is_empty() => return Some(TerminalEvent::Validate),
                            (_, _) => (),
                        };
                        Some(TerminalEvent::Other {
                            code: key.code,
                            modifiers: key.modifiers,
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
