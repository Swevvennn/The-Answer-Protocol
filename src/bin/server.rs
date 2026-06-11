// enum OtherAction {
//     Bind(Result<(), std::io::Error>),
//     Client(Result<tap::network::Client, std::io::Error>),
// }

// type Action = tap::cli::Action<OtherAction>;

// struct Cli {
//     sleeper: tap::utils::Sleeper,
//     input: tap::cli::Input,
//     messages: tap::utils::Shared<tap::cli::Messages>,
//     server: tap::network::Server,
//     game: tap::utils::Shared<tap::game::Game>,
// }

// impl tap::cli::Wrapper for Cli {
//     type OtherAction = OtherAction;

//     fn new() -> Self {
//         Self {
//             sleeper: tap::utils::Sleeper::default(),
//             input: tap::cli::Input::default(),
//             messages: tap::utils::Shared::new(tap::cli::Messages::default()),
//             server: tap::network::Server::default(),
//             game: tap::utils::Shared::new(tap::game::Game::default()),
//         }
//     }

//     async fn draw(&self, terminal: &mut tap::cli::Terminal) {
//         let messages = self.messages.lock().await;
//         terminal.update(|frame| {
//             let chunks = ratatui::layout::Layout::default()
//                 .direction(ratatui::layout::Direction::Vertical)
//                 .constraints([
//                     ratatui::layout::Constraint::Length(4),
//                     ratatui::layout::Constraint::Min(1),
//                     ratatui::layout::Constraint::Length(3),
//                 ])
//                 .split(frame.area());
//             frame.render_widget(
//                 ratatui::widgets::Paragraph::new(
//                     format!(
//                         "Listening at: {} ({})\nClients number: {}",
//                         if self.server.addr.is_empty() { "?" } else { &self.server.addr },
//                         self.server.state,
//                         "?",
//                     )
//                 )
//                     .block(
//                         ratatui::widgets::Block::default()
//                             .borders(ratatui::widgets::Borders::ALL)
//                     ),
//                 chunks[0],
//             );
//             frame.render_widget(
//                 ratatui::widgets::Paragraph::new(messages.to_string())
//                     .block(
//                         ratatui::widgets::Block::default()
//                             .borders(ratatui::widgets::Borders::ALL)
//                     )
//                     .scroll((messages.messages.len().saturating_sub(chunks[1].height.saturating_sub(2) as usize) as u16, 0)),
//                 chunks[1],
//             );
//             frame.render_widget(
//                 ratatui::widgets::Paragraph::new(format!("> {}", self.input))
//                     .block(
//                         ratatui::widgets::Block::default()
//                             .title(
//                                 match self.server.state {
//                                     tap::network::ServerState::Binded => "Press Ctrl + C to exit",
//                                     _ => "Enter a binding address (<IPv4>:<port>)",
//                                 }
//                             )
//                             .borders(ratatui::widgets::Borders::ALL),
//                     ), 
//                 chunks[2],
//             );
//         });
//     }

//     async fn select(&mut self, terminal: &mut tap::cli::Terminal) -> Option<Action> {
//         tokio::select! {
//             _ = self.sleeper.wait() => None,
//             event = terminal.read(&mut self.input) => {
//                 match event {
//                     Some(event) => match event {
//                         tap::cli::TerminalEvent::Interrupted => Some(Action::Interrupt),
//                         tap::cli::TerminalEvent::Validate => Some(Action::Validate),
//                         _ => None,
//                     }
//                     _ => None,
//                 }
//             }
//             action = async {
//                 if matches!(self.server.state, tap::network::ServerState::Binded) {
//                     Some(Action::Other(OtherAction::Client(self.server.accept().await)))
//                 } else if self.server.addr.is_empty() {
//                     tap::utils::Waiter::block().await;
//                     None
//                 } else {
//                     Some(Action::Other(OtherAction::Bind(self.server.bind().await)))
//                 }
//             } => action,
//         }
//     }

//     async fn process(&mut self, action: Action) -> Result<(), std::io::Error> {
//         match action {
//             Action::Other(OtherAction::Bind(r)) => match r {
//                 Ok(()) => {
//                     self.messages.lock().await.log(tap::cli::Message::Head(format!(
//                         "Server listening on {}",
//                         self.server.addr,
//                     )));
//                 }
//                 Err(e) => {
//                     self.messages.lock().await.log(tap::cli::Message::error(e));
//                     self.server.addr = String::new();
//                 }
//             }
//             Action::Other(OtherAction::Client(r)) => match r {
//                 Ok(client) => Client::spawn(&self, client),
//                 Err(e) => {
//                     self.messages.lock().await.log(tap::cli::Message::error(e));
//                     self.server.addr = String::new();
//                 }
//             }
//             Action::Validate if !matches!(self.server.state, tap::network::ServerState::Binded) => {
//                 self.server.addr = self.input.consume();
//                 self.messages.lock().await.log(tap::cli::Message::Info(format!(
//                     "trying to bind on '{}'",
//                     self.server.addr,
//                 )));
//             }
//             _ => (),
//         };
//         Ok(())
//     }

//     async fn update(&mut self) {
//         if matches!(self.server.state, tap::network::ServerState::Terminated) {
//             self.messages.lock().await.log(tap::cli::Message::Head("Server closed".to_string()));
//             self.server.state = tap::network::ServerState::Disconnected;
//         }
//     }
// }

// struct Client {
//     messages: tap::utils::Shared<tap::cli::Messages>,
//     awaker: tap::utils::Awaker,
//     player: tap::game::Player,
//     game: tap::utils::Shared<tap::game::Game>,
// }

// impl Client {
//     pub fn new(cli: &Cli, client: tap::network::Client) -> Self {
//         Self {
//             messages: cli.messages.clone(),
//             awaker: cli.sleeper.new_awaker(),
//             player: tap::game::Player::new(client),
//             game: cli.game.clone(),
//         }
//     }

//     pub fn spawn(cli: &Cli, client: tap::network::Client) {
//         let mut client = Self::new(cli, client);
//         tokio::spawn(async move {
//             if let Err(e) = client.run().await {
//                 client.messages.lock().await.log(tap::cli::Message::Error(format!(
//                     "client {} error: {}",
//                     client.player.client.addr,
//                     e,
//                 )));
//             }
//             client.messages.lock().await.log(tap::cli::Message::Info(format!(
//                 "client {} disconnected",
//                 client.player.client.addr,
//             )));
//         });
//     }

//     pub async fn run(&mut self) -> Result<(), std::io::Error> {
//         self.messages.lock().await.log(tap::cli::Message::Info(format!(
//             "new client connected {}",
//             self.player.client.addr,
//         )));
//         self.awaker.wake().await;
//         self.player.client.write_message(&tap::messages::Message::Response(tap::messages::Response {
//             payload: tap::messages::Payload::new(&[
//                 tap::messages::PayloadKind::String("hello".to_string()),
//                 tap::messages::PayloadKind::KeyValue {
//                     key: "proto".to_string(),
//                     value: "1".to_string(),
//                 },
//             ]),
//         })).await?;
//         loop {
//             if let Some(message) = self.player.client.read().await? {
//                 self.messages.lock().await.log(tap::cli::Message::Network {
//                     from: self.player.client.addr.clone(),
//                     to: "S".to_string(),
//                     message: message.to_string(),
//                 });
//                 self.awaker.wake().await;
//             }
//         }
//         Ok(())
//     }
// }


use clap::Parser;

#[derive(Parser)]
#[command(about = "A Multi-User Dungeon server which use the TAP protocol")]
struct Args {
    world: String,

    /// The server binding ip address
    #[arg(long, short)]
    ip: Option<String>,

    /// The server binding port
    #[arg(long, short)]
    port: Option<String>,
}

struct Cli {
    server: tap::network::Server,
    game: tap::utils::Shared<tap::game::Game>,
}

impl Cli {
    pub async fn start() -> Result<(), std::io::Error> {
        let args = Args::parse();
        let world = match tap::game::World::new(&args.world) {
            Ok(v) => v,
            Err(e) => return Err(std::io::Error::new(
                e.kind(),
                format!("failed to load world data: {e}"),
            )),
        };
        let ip = match args.ip {
            Some(v) => v,
            None => "127.0.0.1".to_string(),
        };
        let port = match args.port {
            Some(v) => v,
            None => "7373".to_string(),
        };
        let mut cli = Cli {
            server: tap::network::Server::new(&format!("{ip}:{port}")),
            game: tap::utils::Shared::new(tap::game::Game::new(world)),
        };
        cli.server.bind().await?;
        tap::cli::logger::info(&format!("Server listening at \x1b[36m{}\x1b[0m", cli.server.addr)).await;
        cli.run().await
    }

    pub async fn run(&mut self) -> Result<(), std::io::Error> {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tap::cli::logger::error("Interrupted").await;
                    self.server.close();
                    break;
                }
                res = self.server.accept() => {
                    match res {
                        Ok(client) => {
                            let game = self.game.clone();
                            let mut player = tap::game::Player::new(client);
                            tokio::spawn(async move {
                                tap::cli::logger::info(&format!(
                                    "New client connected (\x1b[36m{}\x1b[0m)",
                                    player.client.addr,
                                )).await;
                                player.run(game).await;
                                tap::cli::logger::info(&format!(
                                    "Client \x1b[36m{}\x1b[0m disconnected",
                                    player.client.addr,
                                )).await;
                            });
                        }
                        Err(e) => return Err(e),
                    };
                }
            };
        }
        tap::cli::logger::info("Server disconnected").await;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    // tap::cli::run::<Cli>().await;
    match Cli::start().await {
        Ok(_) => (),
        Err(e) => tap::cli::logger::error(&format!("{e}")).await,
    };
}


//                     // tokio::spawn(async move {
//                     //     messages.lock().await.log(tap::cli::Message::Info(format!(
//                     //         "new client connected {}",
//                     //         client.addr,
//                     //     )));
//                     //     awaker.wake().await;
//                     //     let _ = client.write("OK hello proto=1\n").await;
//                     //     loop {
//                     //         match client.read().await {
//                     //             Ok(None) => (),
//                     //             Ok(Some(v)) => {
//                     //                 messages.lock().await.log(tap::cli::Message::Network {
//                     //                     from: client.addr.clone(),
//                     //                     to: "S".to_string(),
//                     //                     message: v.to_string(),
//                     //                 });
//                     //                 awaker.wake().await;
//                     //                 let _ = client.write("OK connected\n").await;
//                     //             },
//                     //             Err(e) => {
//                     //                 messages.lock().await.log(tap::cli::Message::Error(format!(
//                     //                     "client {} error: {}",
//                     //                     client.addr,
//                     //                     e,
//                     //                 )));
//                     //                 break;
//                     //             }
//                     //         };
//                     //     }
//                     //     messages.lock().await.log(tap::cli::Message::Info(format!(
//                     //         "client {} disconnected",
//                     //         client.addr,
//                     //     )));
//                     //     awaker.wake().await;
//                     // });