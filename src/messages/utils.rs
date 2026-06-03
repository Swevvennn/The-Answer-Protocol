use std::fmt;

pub fn write_vec(f: &mut fmt::Formatter<'_>, mut v: Vec<String>) -> fmt::Result {
    v.retain(|s| !s.is_empty());
    write!(f, "{}", v.join(" "))
}
