use futures::StreamExt;

pub type KeyCode = crossterm::event::KeyCode;
pub type KeyModifiers = crossterm::event::KeyModifiers;

pub enum InputEvent {
    Input,
    Interrupted,
    Other {
        code: KeyCode,
        modifiers: KeyModifiers,
    },
    Validate,
}

pub struct Input {
    pub input: String,
    events: crossterm::event::EventStream,
}

impl Input {
    pub fn new() -> Self {
        let _ = crossterm::terminal::enable_raw_mode();
        Self {
            input: String::new(),
            events: crossterm::event::EventStream::new(),
        }
    }

    pub fn consume(&mut self) -> String {
        let input = self.input.clone();
        self.input.clear();
        input
    }

    pub async fn read(&mut self) -> Option<InputEvent> {
        match self.events.next().await {
            Some(Ok(event)) => {
                match event {
                    crossterm::event::Event::Key(key) if key.kind == crossterm::event::KeyEventKind::Press => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(InputEvent::Interrupted),
                            (KeyCode::Char(c), _) => {
                                self.input.push(c);
                                Some(InputEvent::Input)
                            }
                            (KeyCode::Backspace, _) => {
                                self.input.pop();
                                Some(InputEvent::Input)
                            }
                            (KeyCode::Enter, _) => Some(InputEvent::Validate),
                            (_, _) => Some(InputEvent::Other {
                                code: key.code,
                                modifiers: key.modifiers,
                            }),
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}
