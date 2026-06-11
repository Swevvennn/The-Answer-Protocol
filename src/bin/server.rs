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
    match Cli::start().await {
        Ok(_) => (),
        Err(e) => tap::cli::logger::error(&format!("{e}")).await,
    };
}
