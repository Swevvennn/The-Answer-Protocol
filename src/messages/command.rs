use crate::messages::Payload;

pub enum CommandKind {

}

pub struct Command {
    pub kind: CommandKind,
    pub payload: Payload,
}
