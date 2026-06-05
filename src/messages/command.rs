use strum::IntoEnumIterator;

use crate::messages::MessageParse;
use crate::messages::Payload;
use crate::messages::utils;

#[derive(strum_macros::EnumIter)]
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

impl std::fmt::Display for CommandKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    fn from_string(s: &str) -> Result<Command, std::io::Error> {
        let mut message = s.to_string();
        for kind in CommandKind::iter() {
            if utils::parse_begin(&mut message, &kind.to_string()) {
                return Ok(Command {
                    kind,
                    payload: match utils::parse_payload(&mut message) {
                        Ok(v) => v,
                        Err(_) => return Err(utils::invalid_input("invalid command")),
                    }
                });
            }
        }
        Err(utils::invalid_input("invalid command"))
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        utils::write_vec(f, vec![
            self.kind.to_string(),
            self.payload.to_string(),
        ])
    }
}
