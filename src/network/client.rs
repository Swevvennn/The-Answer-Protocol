use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::messages::{Message, MessageParse};

pub struct Client {
    pub addr: String,
    stream: TcpStream,
}

impl Client {
    pub fn new(addr: SocketAddr, stream: TcpStream) -> Self {
        Self {
            addr: addr.to_string(),
            stream,
        }
    }

    pub async fn connect(addr: &str) -> Result<Self, Error> {
        match TcpStream::connect(addr).await {
            Ok(v) => Ok(Self {
                addr: addr.to_string(),
                stream: v,
            }),
            Err(e) => Err(Error::new(
                ErrorKind::HostUnreachable,
                format!("connection to {addr} failed: {e}"),
            )),
        }
    }

    pub async fn read(&mut self) -> Result<Option<Message>, Error> {
        let mut remaining = String::new();
        let mut buffer = [0u8; 1024];
        match self.stream.read(&mut buffer).await {
            Ok(0) => Err(Error::new(
                ErrorKind::ConnectionAborted,
                "read failed: connection closed",
            )),
            Ok(v) => {
                remaining += &String::from_utf8_lossy(&buffer[..v]).to_string();
                match remaining.find('\n') {
                    None => Ok(None),
                    Some(v) => {
                        match Message::from_string(&remaining.drain(..v).collect::<String>()) {
                            Ok(v) => Ok(Some(v)),
                            Err(e) => Err(Error::other(format!("message parsing failed: {e}"))),
                        }
                    }
                }
            },
            Err(e) => Err(Error::new(
                e.kind(),
                format!("read failed: {e}"),
            )),
        }
    }

    pub async fn write(&mut self, s: &str) -> Result<(), Error> {
        match self.stream.write_all(s.as_bytes()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("write failed: {e}"),
            )),
        }
    }
}
