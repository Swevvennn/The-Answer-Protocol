use std::fmt;

use crate::messages::Payload;
use crate::messages::utils::write_vec;

pub enum CommandKind {

}

impl fmt::Display for CommandKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            _ => write!(f, "CHAT"),
        }
    }
}

pub struct Command {
    pub kind: CommandKind,
    pub payload: Payload,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_vec(f, vec![
            self.kind.to_string(),
            self.payload.to_string(),
        ])
    }
}
