use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    buffer: String,
}

impl Client {
    pub fn new(addr: std::net::SocketAddr, stream: tokio::net::TcpStream) -> Self {
        Self {
            state: ClientState::Connected,
            addr: addr.to_string(),
            stream: Some(stream),
            buffer: String::new(),
        }
    }

    pub fn close(&mut self) {
        self.state = ClientState::Terminated;
        self.stream = None;
        self.buffer = String::new();
    }

    pub async fn connect(&mut self) -> Result<(), std::io::Error> {
        if self.stream.is_some() {
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

    pub async fn read(&mut self) -> Result<Option<crate::messages::Message>, std::io::Error> {
        match self.extract_message() {
            Ok(None) => (),
            Err(e) => {
                self.close();
                return Err(e);
            }
            r => return r,
        };
        let mut buffer = [0u8; 1024];
        match &mut self.stream {
            Some(stream) => match match stream.read(&mut buffer).await {
                Ok(0) => Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "read failed: connection closed",
                )),
                Ok(v) => {
                    self.buffer += &String::from_utf8_lossy(&buffer[..v]);
                    self.extract_message()
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
                    Err(std::io::Error::other(format!("write failed: {e}")))
                }
            }
            None => Err(std::io::Error::other("not connected"))
        }
    }

    pub async fn write_message(&mut self, message: &crate::messages::Message) -> Result<(), std::io::Error> {
        self.write(&format!("{message}\n")).await
    }

    fn extract_message(&mut self) -> Result<Option<crate::messages::Message>, std::io::Error> {
        match self.buffer.find('\n') {
            Some(v) => {
                let message = self.buffer.drain(..v).collect::<String>();
                self.buffer.remove(0);
                match crate::messages::Message::from_string(&message) {
                    Ok(v) => Ok(Some(v)),
                    Err(e) => Err(std::io::Error::other(format!("message parsing failed: {e}"))),
                }
            }
            None => Ok(None),
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            state: ClientState::Disconnected,
            addr: String::new(),
            stream: None,
            buffer: String::new(),
        }
    }
}
