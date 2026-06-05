use crate::messages::MessageParse;
use crate::messages::Payload;
use crate::messages::utils;

pub struct Response {
    pub payload: Payload,
}

impl MessageParse for Response {
    fn from_string(s: &str) -> Result<Response, std::io::Error> {
        let mut message = s.to_string();
        if !utils::parse_begin(&mut message, "OK") {
            return Err(utils::invalid_input("not a response"));
        }
        Ok(Response {
            payload: match utils::parse_payload(&mut message) {
                Ok(v) => v,
                Err(_) => return Err(utils::invalid_input("invalid response")),
            }
        })
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        utils::write_vec(f, vec![
            "OK".to_string(),
            self.payload.to_string(),
        ])
    }
}
