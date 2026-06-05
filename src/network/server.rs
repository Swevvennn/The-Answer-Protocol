use crate::network::Client;

pub struct Server {
    pub addr: String,
    listener: tokio::net::TcpListener,
}

impl Server {
    pub async fn new(addr: &str) -> Result<std::sync::Arc<Self>, std::io::Error> {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(v) => Ok(std::sync::Arc::new(Self {
                addr: addr.to_string(),
                listener: v,
            })),
            Err(e) => Err(std::io::Error::new(
                e.kind(),
                format!("bind to {addr} failed: {e}")
            )),
        }
    }

    pub async fn run(self: std::sync::Arc<Self>) {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    let server = std::sync::Arc::clone(&self);
                    tokio::spawn(async move {
                        server
                            .handle_client(Client::new(
                                addr,
                                stream
                            ))
                            .await
                    });
                },
                Err(e) => eprintln!("Error: accept failed: {e}"),
            }
        }
    }

    async fn handle_client(&self, mut client: Client) {
        println!("New client {}", client.addr);
        loop {
            match client.read().await {
                Ok(None) => (),
                Ok(Some(v)) => {
                    println!("Message from {}: {}", client.addr, v);
                    client.write("OK\n").await;
                },
                Err(e) => {
                    eprintln!("Client {} error: {}", client.addr, e);
                    break;
                }
            };
        }
        println!("Client {} disconnected", client.addr);
    }
}
