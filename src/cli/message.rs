pub enum Message {
    Error(String),
    Head(String),
    Incoming(String),
    Info(String),
    Outgoing(String),
    Blank,
}

impl Message {
    pub fn error(e: std::io::Error) -> Self {
        Self::Error(e.to_string())
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error(v) => write!(f, "Error: {v}"),
            Self::Head(v) => write!(f, "\n===== {v} =====\n"),
            Self::Incoming(v) => write!(f, "S -> C: {v}"),
            Self::Info(v) => write!(f, "Info: {v}"),
            Self::Outgoing(v) => write!(f, "C -> S: {v}"),
            Self::Blank => write!(f, ""),
        }
    }
}
