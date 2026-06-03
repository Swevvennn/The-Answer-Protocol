use std::fmt;
use std::io::Error;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::messages::Payload;
use crate::messages::utils::{invalid_input, write_vec};

#[derive(EnumIter)]
pub enum CommandKind {
    Attack,
    Chat,
    Connect,
    Drop,
    GroupCreate,
    GroupInvite,
    GroupJoin,
    GroupLeave,
    Inventory,
    Look,
    Move,
    Quest,
    Quests,
    Quit,
    Status,
    Take,
    Talk,
    Who,
}

impl fmt::Display for CommandKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Attack => write!(f, "ATTACK"),
            Self::Chat => write!(f, "CHAT"),
            Self::Connect => write!(f, "CONNECT"),
            Self::Drop => write!(f, "DROP"),
            Self::GroupCreate => write!(f, "GROUP CREATE"),
            Self::GroupInvite => write!(f, "GROUP INVITE"),
            Self::GroupJoin => write!(f, "GROUP JOIN"),
            Self::GroupLeave => write!(f, "GROUP LEAVE"),
            Self::Inventory => write!(f, "INVENTORY"),
            Self::Look => write!(f, "LOOK"),
            Self::Move => write!(f, "MOVE"),
            Self::Quest => write!(f, "QUEST"),
            Self::Quests => write!(f, "QUESTS"),
            Self::Quit => write!(f, "QUIT"),
            Self::Status => write!(f, "STATUS"),
            Self::Take => write!(f, "TAKE"),
            Self::Talk => write!(f, "TALK"),
            Self::Who => write!(f, "WHO"),
        }
    }
}

pub struct Command {
    pub kind: CommandKind,
    pub payload: Payload,
}

impl Command {
    pub fn from_string(str: String) -> Result<Command, Error> {
        for kind in CommandKind::iter() {
            let kind_str = kind.to_string();
            if str.starts_with(&kind_str) {
                if str.len() > kind_str.len() && let Some(c) = str.chars().nth(kind_str.len()) {
                    if c != ' ' {
                        return Err(invalid_input("invalid command"));
                    }
                }
                let payload = match Payload::from_string(str.chars().skip(kind_str.len() + 1).collect()) {
                    Ok(v) => v,
                    Err(_) => return Err(invalid_input("invalid command")),
                };
                return Ok(Command { kind, payload });
            }
        }
        Err(invalid_input("unknown command"))
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_vec(f, vec![
            self.kind.to_string(),
            self.payload.to_string(),
        ])
    }
}
