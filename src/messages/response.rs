pub struct Response {
    pub payload: crate::messages::Payload,
}

impl Response {
    pub fn from_string(s: &str) -> Result<Response, std::io::Error> {
        let mut message = s.to_string();
        if !crate::messages::utils::parse_begin(&mut message, "OK") {
            return Err(crate::utils::invalid_input("not a response"));
        }
        Ok(Response {
            payload: match crate::messages::utils::parse_payload(&mut message) {
                Ok(v) => v,
                Err(_) => return Err(crate::utils::invalid_input("invalid response")),
            }
        })
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::messages::utils::write_vec(f, vec![
            "OK".to_string(),
            self.payload.to_string(),
        ])
    }
}
