use crate::network::Client;

pub enum ServerState {
    Binded,
    Disconnected,
    Terminated,
}

impl std::fmt::Display for ServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binded => write!(f, "Binded"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

pub struct Server {
    pub state: ServerState,
    pub addr: String,
    listener: Option<tokio::net::TcpListener>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            state: ServerState::Disconnected,
            addr: String::new(),
            listener: None,
        }
    }

    pub fn close(&mut self) {
        self.state = ServerState::Terminated;
        self.listener = None;
    }

    pub async fn bind(&mut self) -> Result<(), std::io::Error> {
        if matches!(self.listener, Some(_)) {
            return Err(std::io::Error::other("already connected"));
        }
        self.listener = match tokio::net::TcpListener::bind(&self.addr).await {
            Ok(v) => Some(v),
            Err(e) => return Err(std::io::Error::new(
                e.kind(),
                format!("bind to '{}' failed: {}", self.addr, e),
            ))
        };
        self.state = ServerState::Binded;
        Ok(())
    }

    pub async fn accept(&mut self) -> Result<Client, std::io::Error> {
        match &self.listener {
            Some(listener) => match listener.accept().await {
                Ok((stream, addr)) => Ok(Client::from_connection(addr, stream)),
                Err(e) => {
                    self.close();
                    Err(std::io::Error::new(
                        e.kind(),
                        format!("accept failed: {e}"),
                    ))
                },
            }
            None => Err(std::io::Error::other("not binded")),
        }
    }

    // pub async fn new(addr: &str) -> Result<std::sync::Arc<Self>, std::io::Error> {
    //     match tokio::net::TcpListener::bind(addr).await {
    //         Ok(v) => Ok(std::sync::Arc::new(Self {
    //             addr: addr.to_string(),
    //             listener: v,
    //         })),
    //         Err(e) => Err(std::io::Error::new(
    //             e.kind(),
    //             format!("bind to {addr} failed: {e}")
    //         )),
    //     }
    // }

    // pub async fn run(self: std::sync::Arc<Self>) {
    //     loop {
    //         match self.listener.accept().await {
    //             Ok((stream, addr)) => {
    //                 let server = std::sync::Arc::clone(&self);
    //                 tokio::spawn(async move {
    //                     server
    //                         .handle_client(Client::from_connection(
    //                             addr,
    //                             stream
    //                         ))
    //                         .await
    //                 });
    //             },
    //             Err(e) => eprintln!("Error: accept failed: {e}"),
    //         }
    //     }
    // }

    // async fn handle_client(&self, mut client: Client) {
    //     println!("New client {}", client.addr);
    //     client.write("OK hello proto=1\n").await;
    //     loop {
    //         match client.read().await {
    //             Ok(None) => (),
    //             Ok(Some(v)) => {
    //                 println!("Message from {}: {}", client.addr, v);
    //                 client.write("OK connected\n").await;
    //             },
    //             Err(e) => {
    //                 eprintln!("Client {} error: {}", client.addr, e);
    //                 break;
    //             }
    //         };
    //     }
    //     println!("Client {} disconnected", client.addr);
    // }
}



// use crate::network::Client;

// pub struct Server {
//     pub addr: String,
//     listener: tokio::net::TcpListener,
// }

// impl Server {
//     pub async fn new(addr: &str) -> Result<std::sync::Arc<Self>, std::io::Error> {
//         match tokio::net::TcpListener::bind(addr).await {
//             Ok(v) => Ok(std::sync::Arc::new(Self {
//                 addr: addr.to_string(),
//                 listener: v,
//             })),
//             Err(e) => Err(std::io::Error::new(
//                 e.kind(),
//                 format!("bind to {addr} failed: {e}")
//             )),
//         }
//     }

//     pub async fn run(self: std::sync::Arc<Self>) {
//         loop {
//             match self.listener.accept().await {
//                 Ok((stream, addr)) => {
//                     let server = std::sync::Arc::clone(&self);
//                     tokio::spawn(async move {
//                         server
//                             .handle_client(Client::from_connection(
//                                 addr,
//                                 stream
//                             ))
//                             .await
//                     });
//                 },
//                 Err(e) => eprintln!("Error: accept failed: {e}"),
//             }
//         }
//     }

//     async fn handle_client(&self, mut client: Client) {
//         println!("New client {}", client.addr);
//         client.write("OK hello proto=1\n").await;
//         loop {
//             match client.read().await {
//                 Ok(None) => (),
//                 Ok(Some(v)) => {
//                     println!("Message from {}: {}", client.addr, v);
//                     client.write("OK connected\n").await;
//                 },
//                 Err(e) => {
//                     eprintln!("Client {} error: {}", client.addr, e);
//                     break;
//                 }
//             };
//         }
//         println!("Client {} disconnected", client.addr);
//     }
// }