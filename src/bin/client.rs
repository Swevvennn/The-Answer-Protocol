// enum Stage {
//     EnteringAddress,
//     WaitingConnection,
//     WaitingGreeting,
//     EnteringUsername,
//     WaitingAuth,
//     EnteringCommand,
//     WaitingResponse,
// }

// enum OtherAction {
//     Connection(Result<(), std::io::Error>),
//     Read(Result<Option<tap::messages::Message>, std::io::Error>),
// }

// type Action = tap::cli::Action<OtherAction>;

// struct Cli {
//     stage: Stage,
//     waiter: tap::utils::Waiter,
//     input: tap::cli::Input,
//     messages: tap::cli::Messages,
//     player: tap::game::Player,
// }

// impl tap::cli::Wrapper for Cli {
//     type OtherAction = OtherAction;

//     fn new() -> Self {
//         Self {
//             stage: Stage::EnteringAddress,
//             waiter: tap::utils::Waiter::default(),
//             input: tap::cli::Input::default(),
//             messages: tap::cli::Messages::default(),
//             player: tap::game::Player::default(),
//         }
//     }

//     async fn draw(&self, terminal: &mut tap::cli::Terminal) {
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
//                         "Server: {} ({})\nUsername: {}",
//                         if self.player.client.addr.is_empty() { "?" } else { &self.player.client.addr },
//                         self.player.client.state,
//                         if self.player.username.is_empty() { "?" } else { &self.player.username },
//                     )
//                 )
//                     .block(
//                         ratatui::widgets::Block::default()
//                             .borders(ratatui::widgets::Borders::ALL)
//                     ),
//                 chunks[0],
//             );
//             frame.render_widget(
//                 ratatui::widgets::Paragraph::new(self.messages.to_string())
//                     .block(
//                         ratatui::widgets::Block::default()
//                             .borders(ratatui::widgets::Borders::ALL)
//                     )
//                     .scroll((self.messages.messages.len().saturating_sub(chunks[1].height.saturating_sub(2) as usize) as u16, 0)),
//                 chunks[1],
//             );
//             frame.render_widget(
//                 ratatui::widgets::Paragraph::new(format!("> {}", self.input))
//                     .block(
//                         ratatui::widgets::Block::default()
//                             .title(
//                                 match self.player.client.state {
//                                     tap::network::ClientState::Connected => "Enter a username",
//                                     tap::network::ClientState::Authenticated => "Enter a command",
//                                     _ => "Enter the server address (<IPv4>:<port>)",
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
//             _ = self.waiter.wait() => Some(Action::Awake),
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
//                 if matches!(self.stage, Stage::WaitingConnection) {
//                     Some(Action::Other(OtherAction::Connection(self.player.client.connect().await)))
//                 } else if matches!(self.player.client.state, tap::network::ClientState::Disconnected | tap::network::ClientState::Terminated) {
//                     tap::utils::Waiter::block().await;
//                     None
//                 } else {
//                     Some(Action::Other(OtherAction::Read(self.player.client.read().await)))
//                 }
//             } => action,
//         }
//     }

//     async fn process(&mut self, action: Action) -> Result<(), std::io::Error> {
//         match action {
//             Action::Other(OtherAction::Connection(r)) => match r {
//                 Ok(_) => {
//                     self.messages.log(tap::cli::Message::Head(format!(
//                         "Connected to {}",
//                         self.player.client.addr,
//                     )));
//                     self.stage = Stage::WaitingGreeting;
//                     self.waiter.begin();
//                 }
//                 Err(e) => {
//                     self.messages.log(tap::cli::Message::error(e));
//                     self.stage = Stage::EnteringAddress;
//                     self.waiter.end();
//                 }
//             }
//             Action::Other(OtherAction::Read(r)) => match r {
//                 Ok(message) => {
//                     if let Some(message) = message {
//                         if match (&self.stage, &message) {
//                             (Stage::WaitingGreeting, tap::messages::Message::Response(message)) => {
//                                 let mut version = String::new();
//                                 match message.payload.extract(&mut [
//                                     tap::messages::PayloadExtractor::String(&mut "hello".to_string()),
//                                     tap::messages::PayloadExtractor::KeyValue {
//                                         key: &mut "proto".to_string(),
//                                         value: &mut version
//                                     }
//                                 ]) {
//                                     Ok(_) => {
//                                         self.stage = Stage::EnteringUsername;
//                                         true
//                                     }
//                                     Err(_) => false,
//                                 }
//                             }
//                             (Stage::WaitingAuth, tap::messages::Message::Error(_)) => {
//                                 self.stage = Stage::EnteringUsername;
//                                 true
//                             }
//                             (Stage::WaitingAuth, tap::messages::Message::Response(message)) => match message.payload.extract(&mut [
//                                 tap::messages::PayloadExtractor::String(&mut "connected".to_string()),
//                             ]) {
//                                 Ok(_) => {
//                                     self.stage = Stage::EnteringCommand;
//                                     true
//                                 }
//                                 _ => false,
//                             }
//                             (Stage::WaitingResponse, tap::messages::Message::Response(_) | tap::messages::Message::Error(_)) => {
//                                 self.stage = Stage::EnteringCommand;
//                                 true
//                             }
//                             (Stage::EnteringCommand, tap::messages::Message::Event(_)) => true,
//                             (_, _) => false,
//                         } {
//                             self.messages.log(tap::cli::Message::Network {
//                                 from: "S".to_string(),
//                                 to: "C".to_string(),
//                                 message: message.to_string(),
//                             });
//                         } else {
//                             self.messages.log(tap::cli::Message::Error(format!("unexpected message received from the server: {message}")));
//                             self.player.client.close();
//                         }
//                         self.waiter.end();
//                     }
//                 }
//                 Err(e) => self.messages.log(tap::cli::Message::error(e)),
//             }
//             Action::Awake => {
//                 self.messages.log(tap::cli::Message::Error("the server is not responding".to_string()));
//                 self.player.client.close();
//             }
//             Action::Validate => {
//                 match &self.stage {
//                     Stage::EnteringAddress => {
//                         self.player.client.addr = self.input.consume();
//                         self.messages.log(tap::cli::Message::Info(format!(
//                             "attempting to connect to '{}'",
//                             self.player.client.addr,
//                         )));
//                         self.stage = Stage::WaitingConnection;
//                         self.waiter.begin();
//                     }
//                     Stage::EnteringUsername => {
//                         self.player.username = self.input.consume();
//                         self.messages.log(tap::cli::Message::Info(format!(
//                             "try to authenticate with username '{}'",
//                             self.player.username,
//                         )));
//                         match self.player.client.write_message(&tap::messages::Message::Command(tap::messages::Command {
//                             kind: tap::messages::CommandKind::Connect,
//                             payload: tap::messages::Payload::new(&[
//                                 tap::messages::PayloadKind::String(self.player.username.clone()),
//                             ]),
//                         })).await {
//                             Ok(_) => {
//                                 self.stage = Stage::WaitingAuth;
//                                 self.waiter.begin();
//                             },
//                             Err(e) => self.messages.log(tap::cli::Message::error(e)),
//                         };
//                     }
//                     Stage::EnteringCommand => {
//                         let input = self.input.consume();
//                         self.messages.log(tap::cli::Message::Network {
//                             from: "C".to_string(),
//                             to: "S".to_string(),
//                             message: input.clone(),
//                         });
//                         match tap::messages::Message::from_str(&input) {
//                             Ok(message) => match self.player.client.write_message(&message).await {
//                                 Ok(_) => {
//                                     self.stage = Stage::WaitingResponse;
//                                     self.waiter.begin();
//                                 },
//                                 Err(e) => self.messages.log(tap::cli::Message::error(e)),
//                             },
//                             Err(_) => self.messages.log(tap::cli::Message::Error("invalid command".to_string())),
//                         }
//                     }
//                     _ => (),
//                 };
//             }
//             _ => (),
//         };
//         Ok(())
//     }

//     async fn update(&mut self) {
//         if matches!(self.player.client.state, tap::network::ClientState::Terminated) {
//             self.messages.log(tap::cli::Message::Head(format!(
//                 "Connection to {} closed",
//                 self.player.client.addr,
//             )));
//             self.player.client.state = tap::network::ClientState::Disconnected;
//             self.stage = Stage::EnteringAddress;
//             self.waiter.end();
//         };
//     }
// }

// #[tokio::main]
// async fn main() {
//     tap::cli::run::<Cli>().await;
// }








use clap::Parser;
use std::io::Write;
use std::str::FromStr;

use tap::messages::{
    Command,
    CommandKind,
    Error,
    Event,
    EventKind,
    EventScope,
    Message,
    Payload,
    PayloadExtractor,
    PayloadJson,
    PayloadKind,
    Response,
};

#[derive(Parser)]
#[command(about = "A Multi-User Dungeon client which use the TAP protocol")]
struct Args {
    /// The server binding ip address
    #[arg(long, short)]
    ip: Option<String>,

    /// The server binding port
    #[arg(long, short)]
    port: Option<String>,

    /// If enable, enter into raw client
    #[arg(long, short, action = clap::ArgAction::SetTrue)]
    raw: bool,
}

#[derive(Default)]
struct Cli {
    waiter: tap::utils::Waiter,
    input: tap::cli::Input,
    client: tap::network::Client,
    player: tap::game::Player,
    command: Option<Command>,
}

impl Cli {
    pub async fn start() -> Option<Error> {
        let args = Args::parse();
        let ip = match args.ip {
            Some(v) => v,
            None => "127.0.0.1".to_string(),
        };
        let port = match args.port {
            Some(v) => v,
            None => "7373".to_string(),
        };
        let mut cli = Cli::default();
        cli.client.addr = format!("{ip}:{port}");
        if cli.client.connect().await.is_err() {
            return Some(Error::ConnectionFailed);
        }
        cli.waiter.begin(3);
        if args.raw {
            cli.run_raw().await
        } else {
            cli.run_friendly().await
        }
    }

    async fn run_raw(&mut self) -> Option<Error> {
        fn clear_line() {
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveToColumn(0),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::CurrentLine),
            );
        }

        fn print_input(cli: &Cli) {
            clear_line();
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::style::Print(format!(
                    "[{} proto={}] {}: ",
                    cli.client.addr,
                    cli.client.proto,
                    if cli.player.username.is_empty() { "?" } else { &cli.player.username },
                )),
                crossterm::style::Print(&cli.input.input),
            );
            let _ = std::io::stdout().flush();
        }

        fn print_out(cli: &Cli, s: &str) {
            clear_line();
            let _ = crossterm::execute!(
                std::io::stdout(),
                crossterm::style::Print(s),
                crossterm::style::Print("\n\n"),
            );
            print_input(cli);
            let _ = std::io::stdout().flush();
        }

        fn print_err(cli: &Cli, error: &Error) {
            clear_line();
            let _ = crossterm::execute!(
                std::io::stderr(),
                crossterm::style::Print(error.to_string()),
                crossterm::style::Print("\n\n"),
            );
            print_input(cli);
            let _ = std::io::stderr().flush();
        }

        print_input(self);
        loop {
            if let Some(e) = tokio::select! {
                _ = self.waiter.wait() => Some(Error::ServerTimeOut),
                event = self.input.read() => {
                    if let Some(event) = event {
                        match event {
                            tap::cli::InputEvent::Interrupted => return None,
                            tap::cli::InputEvent::Validate if !self.input.input.is_empty() && !self.waiter.is_waiting() => {
                                print_out(self, &self.input.input);
                                match Command::from_str(&self.input.consume()) {
                                    Ok(command) => {
                                        self.waiter.begin(3);
                                        self.command = Some(command.clone());
                                        if let Some(writer) = &self.client.writer {
                                            match writer.write_message(&Message::Command(command)).await {
                                                Ok(_) => None,
                                                Err(_) => Some(Error::SendFailed),
                                            }
                                        } else {
                                            Some(Error::SendFailed)
                                        }
                                    }
                                    Err(_) => Some(Error::NotACommand),
                                }
                            }
                            tap::cli::InputEvent::Input => {
                                print_input(self);
                                None
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                message = self.client.reader.read() => {
                    match message {
                        Ok(Some(message)) => match Message::from_str(&message) {
                            Ok(message) => {
                                self.waiter.end();
                                match message {
                                    Message::Error(error) => Some(error),
                                    Message::Response(response) => {
                                        let r = self.process_response(&response);
                                        print_out(self, &response.to_string());
                                        r
                                    }
                                    Message::Event(event) => {
                                        print_out(self, &event.to_string());
                                        None
                                    }
                                    Message::Command(_) => Some(Error::UnexpectedServerResponse),
                                }
                            }
                            Err(_) => Some(Error::UnexpectedServerResponse),
                        }
                        Ok(None) => break,
                        Err(_) => Some(Error::ConnectionClosed),
                    }
                }
            } {
                if e.is_fatal() {
                    clear_line();
                    return Some(e);
                } else {
                    print_err(self, &e);
                }
            }
        }
        None
    }

    async fn run_friendly(&mut self) -> Option<Error> {
        None
    }

    fn process_response(&mut self, response: &Response) -> Option<Error> {
        match &self.command {
            Some(command) => match command.kind {
                CommandKind::Connect => match response.payload.extract(&mut [
                    PayloadExtractor::String(&mut "connected".to_string()),
                ]) {
                    Ok(_) => {
                        self.client.state = tap::network::ClientState::Authenticated;
                        self.player.username.clear();
                        if command.payload.extract(&mut [
                            PayloadExtractor::String(&mut self.player.username),
                        ]).is_err() {
                            return Some(Error::UnexpectedServerResponse);
                        }
                    },
                    Err(_) => return Some(Error::UnexpectedServerResponse),
                }
                _ => (),
            }
            None => {
                self.client.proto.clear();
                match response.payload.extract(&mut [
                    PayloadExtractor::String(&mut "hello".to_string()),
                    PayloadExtractor::KeyValue {
                        key: &mut "proto".to_string(),
                        value: &mut self.client.proto,
                    }
                ]) {
                    Ok(_) => (),
                    Err(_) => return Some(Error::UnexpectedServerResponse),
                }
            },
        };
        self.command = None;
        None
    }
}

#[tokio::main]
async fn main() {
    if let Some(e) = Cli::start().await {
        eprintln!("{e}");
    };
}
