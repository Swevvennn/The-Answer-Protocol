use std::sync::Arc;
use tokio::net::TcpListener;

use crate::network::Client;

pub struct Server {
    pub addr: String,
    listener: TcpListener,
}

impl Server {
    pub async fn new(addr: &str) -> Arc<Self> {
        let listener = TcpListener::bind(addr)
            .await
            .expect(&format!("bind to {} failed", addr));
        Arc::new(Self {
            addr: addr.to_string(),
            listener,
        })
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            let (stream, addr) = self.listener
                .accept()
                .await
                .expect("accept failed");
            let server = Arc::clone(&self);
            tokio::spawn(async move {
                server
                    .handle_client(Client::new(addr, stream))
                    .await
            });
        }
    }

    async fn handle_client(&self, mut client: Client) {
        println!("New client {}", client.addr);
        loop {
            let message = match client.read().await {
                Ok(Some(message)) => message,
                Ok(None) => break,
                Err(e) => {
                    eprintln!("Client {} error: {}", client.addr, e);
                    break;
                }
            };
            println!("Message from {}: {}", client.addr, message);
        }
        println!("Client {} disconnected", client.addr);
    }
}
