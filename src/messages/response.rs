use std::fmt;

use crate::messages::Payload;
use crate::messages::utils::write_vec;

pub struct Response {
    pub payload: Payload,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_vec(f, vec![
            "OK".to_string(),
            self.payload.to_string(),
        ])
    }
}
