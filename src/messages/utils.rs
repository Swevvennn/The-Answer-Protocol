use std::fmt;
use std::io::{Error, ErrorKind};

pub fn write_vec(f: &mut fmt::Formatter<'_>, mut v: Vec<String>) -> fmt::Result {
    v.retain(|s| !s.is_empty());
    write!(f, "{}", v.join(" "))
}

pub fn invalid_input(str: &str) -> Error {
    Error::new(
        ErrorKind::InvalidInput,
        str,
    )
}
