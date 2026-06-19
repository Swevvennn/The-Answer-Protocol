use clap::Parser;
use std::io::Write;
use std::str::FromStr;

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
    command: Option<tap::messages::Command>,
}

impl Cli {
    pub async fn start() -> Option<tap::messages::Error> {
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
            return Some(tap::messages::Error::ConnectionFailed);
        }
        cli.waiter.begin(3);
        if args.raw {
            cli.run_raw().await
        } else {
            cli.run_friendly().await
        }
    }

    async fn run_raw(&mut self) -> Option<tap::messages::Error> {
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

        fn print_err(cli: &Cli, error: &tap::messages::Error) {
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
                _ = self.waiter.wait() => Some(tap::messages::Error::ServerTimeOut),
                event = self.input.read() => {
                    if let Some(event) = event {
                        match event {
                            tap::cli::InputEvent::Interrupted => return None,
                            tap::cli::InputEvent::Validate if !self.input.input.is_empty() && !self.waiter.is_waiting() => {
                                print_out(self, &self.input.input);
                                match tap::messages::Command::from_str(&self.input.consume()) {
                                    Ok(command) => {
                                        self.waiter.begin(3);
                                        self.command = Some(command.clone());
                                        if let Some(writer) = &self.client.writer {
                                            match writer.write_message(&tap::messages::Message::Command(command)).await {
                                                Ok(_) => None,
                                                Err(_) => Some(tap::messages::Error::SendFailed),
                                            }
                                        } else {
                                            Some(tap::messages::Error::SendFailed)
                                        }
                                    }
                                    Err(_) => Some(tap::messages::Error::NotACommand),
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
                        Ok(Some(message)) => match tap::messages::Message::from_str(&message) {
                            Ok(message) => {
                                self.waiter.end();
                                match message {
                                    tap::messages::Message::Error(error) => Some(error),
                                    tap::messages::Message::Response(response) => {
                                        let r = self.process_response(&response);
                                        print_out(self, &response.to_string());
                                        r
                                    }
                                    tap::messages::Message::Event(event) => {
                                        print_out(self, &event.to_string());
                                        None
                                    }
                                    tap::messages::Message::Command(_) => Some(tap::messages::Error::UnexpectedServerResponse),
                                }
                            }
                            Err(_) => Some(tap::messages::Error::UnexpectedServerResponse),
                        }
                        Ok(None) => break,
                        Err(_) => Some(tap::messages::Error::ConnectionClosed),
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

    async fn run_friendly(&mut self) -> Option<tap::messages::Error> {
        None
    }

    fn process_response(&mut self, response: &tap::messages::Response) -> Option<tap::messages::Error> {
        match &self.command {
            Some(command) if matches!(command.kind, tap::messages::CommandKind::Connect) => match response.payload.extract(&mut [
                tap::messages::PayloadExtractor::String(&mut "connected".to_string()),
            ]) {
                Ok(_) => {
                    self.client.state = tap::network::ClientState::Authenticated;
                    self.player.username.clear();
                    if command.payload.extract(&mut [
                        tap::messages::PayloadExtractor::String(&mut self.player.username),
                    ]).is_err() {
                        return Some(tap::messages::Error::UnexpectedServerResponse);
                    }
                },
                Err(_) => return Some(tap::messages::Error::UnexpectedServerResponse),
            }
            None => {
                self.client.proto.clear();
                match response.payload.extract(&mut [
                    tap::messages::PayloadExtractor::String(&mut "hello".to_string()),
                    tap::messages::PayloadExtractor::KeyValue {
                        key: &mut "proto".to_string(),
                        value: &mut self.client.proto,
                    }
                ]) {
                    Ok(_) => (),
                    Err(_) => return Some(tap::messages::Error::UnexpectedServerResponse),
                }
            },
            _ => (),
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
