use clap::Parser;
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
    game: tap::utils::Shared<tap::game::GameState>,
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
            game: tap::utils::Shared::new(tap::game::GameState::new(world)),
        };
        cli.server.bind().await?;
        Logger::info(&format!("Server listening at \x1b[36m{}\x1b[0m", cli.server.addr)).await;
        cli.run().await
    }

    async fn run(&mut self) -> Result<(), std::io::Error> {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    Logger::error("Interrupted").await;
                    self.server.close();
                    break;
                }
                res = self.server.accept() => {
                    match res {
                        Ok(client) => {
                            let game = self.game.clone();
                            tokio::spawn(async move {
                                Self::run_client(client, game).await;
                            });
                        }
                        Err(e) => return Err(e),
                    };
                }
            };
        }
        Logger::info("Server disconnected").await;
        Ok(())
    }

    async fn run_client(mut client: tap::network::Client, game: tap::utils::Shared<tap::game::GameState>) {
        let mut username = String::new();
        Logger::info(&format!(
            "New client connected (\x1b[36m{}\x1b[0m)",
            client.addr,
        )).await;
        Self::send_to(
            &client,
            &username,
            &Message::Response(Response {
                payload: Payload::new(&[
                    PayloadKind::String("hello".to_string()),
                    PayloadKind::KeyValue {
                        key: "proto".to_string(),
                        value: "1".to_string(),
                    }
                ])
            }),
        ).await;
        loop {
            match client.reader.read().await {
                Ok(Some(message)) => {
                    Logger::info(&format!(
                        "From \x1b[0;35m{}\x1b[0;0m (\x1b[0;36m{}\x1b[0;0m): {}",
                        if username.is_empty() { "?" } else { &username },
                        client.addr,
                        message,
                    )).await;
                    match Message::from_str(&message) {
                        Ok(Message::Command(command)) => {
                            if command.kind.requires_auth() && !matches!(client.state, tap::network::ClientState::Authenticated) {
                                Self::send_to(&client, &username, &Message::Error(Error::NotAuthenticated)).await;
                            } else if matches!(command.kind, CommandKind::Connect) && matches!(client.state, tap::network::ClientState::Authenticated) {
                                Self::send_to(&client, &username, &Message::Error(Error::AlreadyAuthenticated)).await;
                            } else {
                                let mut game = game.lock().await;
                                let message = Self::process_command(&mut client, &mut username, &mut game, &command).await;
                                Self::send_to(&client, &username, &message).await;
                                if let Message::Response(response) = message && (
                                    client.is_open() &&
                                    response.payload.args.len() == 1 &&
                                    if let PayloadKind::String(s) = &response.payload.args[0] {
                                        s == "bye"
                                    } else {
                                        false
                                    }
                                ) {
                                    client.close();
                                }
                            }
                        }
                        _ => Self::send_to(&client, &username, &Message::Error(Error::NotACommand)).await,
                    };
                }
                Ok(None) => break,
                Err(e) => {
                    Logger::error(&format!(
                        "client \x1b[36m{}\x1b[0m: {}",
                        client.addr,
                        e,
                    )).await;
                    if client.is_open() {
                        Self::send_to(&client, &username, &Message::Error(Error::NotACommand)).await;
                    } else {
                        break;
                    }
                }
            };
            if !client.is_open() {
                break;
            }
        }
        if !username.is_empty() {
            let mut game = game.lock().await;
            Self::send_event(
                &username,
                &game,
                &Event {
                    scope: EventScope::Stats,
                    kind: EventKind::Players,
                    payload: Payload::new(&[
                        PayloadKind::KeyValue {
                            key: "players".to_string(),
                            value: (game.players.len() - 1).to_string(),
                        },
                    ]),
                },
            ).await;
            game.players.remove(&username);
        }
        Logger::info(&format!(
            "Client \x1b[36m{}\x1b[0m disconnected",
            client.addr,
        )).await;
    }

    async fn process_command(client: &mut tap::network::Client, username: &mut String, game: &mut tap::game::GameState, command: &Command) -> Message {
        match command.kind {
            CommandKind::Chat => {
                let mut scope = EventScope::Global;
                let mut message = String::new();
                if let Err(_) = command.payload.extract(&mut [
                    PayloadExtractor::Keyword(&mut scope),
                    PayloadExtractor::String(&mut message),
                ]) {
                    Message::Error(Error::InvalidArguments)
                } else if matches!(scope, EventScope::Stats) {
                    Message::Error(Error::InvalidArguments)
                } else {
                    Self::send_event(
                        username,
                        game,
                        &Event {
                            scope,
                            kind: EventKind::Chat,
                            payload: Payload::new(&[
                                PayloadKind::String(username.clone()),
                                PayloadKind::String(message),
                            ]),
                        },
                    ).await;
                    Message::Response(Response::default())
                }
            }
            CommandKind::Connect => {
                username.clear();
                if let Err(_) = command.payload.extract(&mut [
                    PayloadExtractor::String(username),
                ]) {
                    Message::Error(Error::InvalidArguments)
                } else if game.players.contains_key(username) {
                    Message::Error(Error::NameInUse)
                } else if username.is_empty() {
                    Message::Error(Error::InvalidName)
                } else {
                    if let Some(writer) = &client.writer {
                        client.state = tap::network::ClientState::Authenticated;
                        game.players.insert(
                            username.clone(),
                            tap::game::Player {
                                username: username.clone(),
                                group: String::new(),
                                room: game.start.clone(),
                                writer: Some(writer.clone()),
                            },
                        );
                        Self::send_event(
                            username,
                            game,
                            &Event {
                                scope: EventScope::Stats,
                                kind: EventKind::Players,
                                payload: Payload::new(&[
                                    PayloadKind::KeyValue {
                                        key: "players".to_string(),
                                        value: game.players.len().to_string(),
                                    },
                                ]),
                            },
                        ).await;
                        Message::Response(Response {
                            payload: Payload::new(&[
                                PayloadKind::String("connected".to_string()),
                            ]),
                        })
                    } else {
                        Message::Error(Error::ServerError)
                    }
                }
            }
            CommandKind::Quit => {
                if command.payload.is_empty() {
                    Message::Response(Response {
                        payload: Payload::new(&[
                            PayloadKind::String("bye".to_string()),
                        ]),
                    })
                } else {
                    Message::Error(Error::InvalidArguments)
                }
            }
            CommandKind::Look => {
                if command.payload.is_empty() {
                    Message::Response(Response {
                        payload: Payload::new(&[
                            PayloadKind::new_json(&game.rooms[&game.players[username].room]),
                        ]),
                    })
                } else {
                    Message::Error(Error::InvalidArguments)
                }
            }
            CommandKind::Move => {
                let mut direction = tap::game::Direction::East;
                if let Err(_) = command.payload.extract(&mut [
                    PayloadExtractor::Keyword(&mut direction),
                ]) {
                    Message::Error(Error::InvalidArguments)
                } else {
                    if let Some(player) = game.players.get_mut(username) {
                        let room = &game.rooms[&player.room].room;
                        if room.exits.contains_key(&direction) {
                            let room = room.exits[&direction].clone();
                            // Self::send_event(
                            //     username,
                            //     game,
                            //     &Event {
                            //         scope: EventScope::Room,
                            //         kind: EventKind::PresenceLeave,
                            //         payload: Payload::new(&[
                            //             PayloadKind::String(username.clone()),
                            //             PayloadKind::String(direction.to_string()),
                            //             PayloadKind::String(room.clone()),
                            //         ]),
                            //     },
                            // ).await;
                            player.room = room;
                            Message::Response(Response {
                                payload: Payload::new(&[
                                    PayloadKind::KeyValue {
                                        key: "room".to_string(),
                                        value: player.room.clone(),
                                    },
                                ]),
                            })
                        } else {
                            Message::Error(Error::NoExit)
                        }
                    } else {
                        Message::Error(Error::ServerError)
                    }
                }
            }
            CommandKind::Who => {
                if command.payload.is_empty() {
                    Message::Response(Response {
                        payload: Payload::new(&[
                            PayloadKind::KeyValue {
                                key: "players".to_string(),
                                value: game.players.len().to_string(),
                            },
                        ]),
                    })
                } else {
                    Message::Error(Error::InvalidArguments)
                }
            }
            _ => Message::Response(Response::default()),
        }
    }

    async fn send_to(client: &tap::network::Client, username: &String, message: &Message) {
        Logger::info(&format!(
            "To \x1b[0;35m{}\x1b[0;0m (\x1b[0;36m{}\x1b[0;0m): {}",
            if username.is_empty() { "?" } else { &username },
            client.addr,
            message,
        )).await;
        if let Some(writer) = &client.writer {
            let _ = writer.write_message(message).await;
        }
    }

    async fn send_event(username: &String, game: &tap::game::GameState, event: &tap::messages::Event) {
        let from = &game.players[username];
        Logger::info(&format!(
            "{} event: {}",
            match event.scope {
                EventScope::Global => "global".to_string(),
                EventScope::Group => format!("group {}", from.group),
                EventScope::Room => format!("room {}", from.room),
                EventScope::Stats => "stats".to_string(),
            },
            event,
        )).await;
        let message = Message::Event(event.clone());
        for (_, player) in &game.players {
            if let Some(writer) = &player.writer {
                if match event.scope {
                    EventScope::Global | EventScope::Stats => true,
                    EventScope::Group if player.group == from.group => true,
                    EventScope::Room if player.room == from.room => true,
                    _ => false, 
                } {
                    let _ = writer.write_message(&message).await;
                }
            }
        }
    }
}

pub enum LogKind {
    Error,
    Info,
    Warning,
}

impl std::fmt::Display for LogKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "\x1b[31mERROR\x1b[0m"),
            Self::Info => write!(f, "\x1b[34mINFO\x1b[0m"),
            Self::Warning => write!(f, "\x1b[33mWARN\x1b[0m"),
        }
    }
}

static LOG_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> = std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

struct Logger;

impl Logger {
    async fn log(kind: LogKind, s: &str) {
        let _guard = LOG_LOCK.lock().await;
        let s = format!(
            "\x1b[90m[{}]\x1b[0m {}: {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            kind,
            s,
        );
        if matches!(kind, LogKind::Error) {
            eprintln!("{s}");
        } else {
            println!("{s}");
        }
    }

    pub async fn error(s: &str) {
        Self::log(LogKind::Error, s).await;
    }

    pub async fn info(s: &str) {
        Self::log(LogKind::Info, s).await;
    }

    pub async fn warning(s: &str) {
        Self::log(LogKind::Warning, s).await;
    }
}

#[tokio::main]
async fn main() {
    match Cli::start().await {
        Ok(_) => (),
        Err(e) => Logger::error(&format!("{e}")).await,
    };
}
