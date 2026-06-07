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
    stream: Option<tokio::net::TcpStream>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            state: ClientState::Disconnected,
            addr: String::new(),
            stream: None,
        }
    }

    pub fn from_connection(addr: std::net::SocketAddr, stream: tokio::net::TcpStream) -> Self {
        Self {
            state: ClientState::Connected,
            addr: addr.to_string(),
            stream: Some(stream),
        }
    }

    pub fn close(&mut self) {
        self.state = ClientState::Terminated;
        self.stream = None;
    }

    pub async fn connect(&mut self) -> Result<(), std::io::Error> {
        if matches!(self.stream, Some(_)) {
            return Err(std::io::Error::other("already connected"));
        }
        self.stream = match tokio::net::TcpStream::connect(&self.addr).await {
            Ok(v) => Some(v),
            Err(e) => return Err(std::io::Error::new(
                e.kind(),
                format!("connection to '{}' failed: {}", &self.addr, e),
            )),
        };
        self.state = ClientState::Connected;
        Ok(())
    }

    pub async fn read(&mut self) -> Result<Option<Message>, std::io::Error> {
        let mut remaining = String::new();
        let mut buffer = [0u8; 1024];
        match &mut self.stream {
            Some(stream) => match match stream.read(&mut buffer).await {
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
            } {
                Ok(v) => Ok(v),
                Err(e) => {
                    self.close();
                    Err(e)
                }
            }
            None => Err(std::io::Error::other("not connected"))
        }
    }

    pub async fn write(&mut self, s: &str) -> Result<(), std::io::Error> {
        match &mut self.stream {
            Some(stream) => match stream.write_all(s.as_bytes()).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    self.close();
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("write failed: {e}"),
                    ))
                }
            }
            None => Err(std::io::Error::other("not connected"))
        }
    }

    pub async fn write_message(&mut self, message: &Message) -> Result<(), std::io::Error> {
        self.write(&format!("{message}\n")).await
    }
}
