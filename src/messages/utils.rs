use crate::messages::MessageParse;
use crate::messages::Payload;

pub fn write_vec(f: &mut std::fmt::Formatter<'_>, mut v: Vec<String>) -> std::fmt::Result {
    v.retain(|s| !s.is_empty());
    write!(f, "{}", v.join(" "))
}

pub fn skip_space(s: &mut String) -> Result<bool, std::io::Error> {
    if parse_begin(s, " ") {
        if s.is_empty() {
            return Err(invalid_input(""));
        }
        return Ok(true);
    }
    Ok(false)
}

pub fn parse_begin(s: &mut String, prefix: &str) -> bool {
    if let Some(rest) = s.strip_prefix(prefix) {
        *s = rest.to_string();
        true
    } else {
        false
    }
}

pub fn parse_payload(s: &mut String) -> Result<Payload, std::io::Error> {
    let space = match skip_space(s) {
        Ok(v) => v,
        Err(_) => return Err(invalid_input("invalid payload")),
    };
    if space {
        match Payload::from_string(&s) {
            Ok(v) => Ok(v),
            Err(_) => Err(invalid_input("invalid payload")),
        }
    } else {
        Ok(Payload { args: Vec::new() })
    }
}

pub fn invalid_input(s: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        s,
    )
}
