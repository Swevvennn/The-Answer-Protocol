use std::fmt;
use std::io::Error;

use crate::messages::MessageParse;
use crate::messages::Payload;
use crate::messages::utils::{invalid_input, parse_begin, parse_payload, write_vec};

pub struct Response {
    pub payload: Payload,
}

impl MessageParse for Response {
    fn from_string(s: &str) -> Result<Response, Error> {
        let mut message = s.to_string();
        if !parse_begin(&mut message, "OK") {
            return Err(invalid_input("not a response"));
        }
        Ok(Response {
            payload: match parse_payload(&mut message) {
                Ok(v) => v,
                Err(_) => return Err(invalid_input("invalid response")),
            }
        })
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_vec(f, vec![
            "OK".to_string(),
            self.payload.to_string(),
        ])
    }
}
