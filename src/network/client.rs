use std::io::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

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

    pub async fn connect(addr: &str) -> Self {
        let stream = TcpStream::connect(addr)
            .await
            .expect(&format!("connection to {} failed", addr));
        Self {
            addr: addr.to_string(),
            stream
        }
    }

    pub async fn read(&mut self) -> Result<Option<String>, Error> {
        let mut buffer = [0u8; 1024];
        let n = self.stream
            .read(&mut buffer)
            .await?;
        if n == 0 { return Ok(None); }
        Ok(Some(String::from_utf8_lossy(&buffer[..n]).to_string()))
    }

    pub async fn write(&mut self, msg: &str) {
        self.stream
            .write_all(msg.as_bytes())
            .await
            .expect("write failed");
    }
}
