use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::messages::{Message, MessageParse};

pub enum ClientState {
    Authenticated,
    Connected,
    Disconnected,
    Terminated,
}

impl std::fmt::Display for ClientState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authenticated => write!(f, "Authenticated"),
            Self::Connected => write!(f, "Connected"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

pub struct Client {
    pub state: ClientState,
    pub addr: String,
    stream: tokio::net::TcpStream,
}

impl Client {
    pub fn new(addr: std::net::SocketAddr, stream: tokio::net::TcpStream) -> Self {
        Self {
            state: ClientState::Connected,
            addr: addr.to_string(),
            stream,
        }
    }

    pub async fn connect(addr: &str) -> Result<Self, std::io::Error> {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(v) => Ok(Self {
                state: ClientState::Connected,
                addr: addr.to_string(),
                stream: v,
            }),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::HostUnreachable,
                format!("connection to {addr} failed: {e}"),
            )),
        }
    }

    pub async fn read(&mut self) -> Result<Option<Message>, std::io::Error> {
        let mut remaining = String::new();
        let mut buffer = [0u8; 1024];
        match self.stream.read(&mut buffer).await {
            Ok(0) => Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionAborted,
                "read failed: connection closed",
            )),
            Ok(v) => {
                remaining += &String::from_utf8_lossy(&buffer[..v]).to_string();
                match remaining.find('\n') {
                    None => Ok(None),
                    Some(v) => {
                        match Message::from_string(&remaining.drain(..v).collect::<String>()) {
                            Ok(v) => Ok(Some(v)),
                            Err(e) => Err(std::io::Error::other(format!("message parsing failed: {e}"))),
                        }
                    }
                }
            },
            Err(e) => Err(std::io::Error::new(
                e.kind(),
                format!("read failed: {e}"),
            )),
        }
    }

    pub async fn write(&mut self, s: &str) -> Result<(), std::io::Error> {
        match self.stream.write_all(s.as_bytes()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("write failed: {e}"),
            )),
        }
    }
}
