use std::fmt;
use std::io::Error;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::messages::MessageParse;
use crate::messages::Payload;
use crate::messages::utils::{invalid_input, parse_begin, parse_payload, write_vec};

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

impl MessageParse for Command {
    fn from_string(mut s: String) -> Result<Command, Error> {
        for kind in CommandKind::iter() {
            if parse_begin(&mut s, &kind.to_string()) {
                return Ok(Command {
                    kind,
                    payload: match parse_payload(&mut s) {
                        Ok(v) => v,
                        Err(_) => return Err(invalid_input("invalid command")),
                    }
                });
            }
        }
        Err(invalid_input("invalid command"))
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
