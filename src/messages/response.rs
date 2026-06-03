use std::fmt;

use crate::messages::Payload;

pub struct Response {
    pub payload: Payload,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OK {}",
            self.payload
        )
    }
}
