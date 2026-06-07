use tap::network::Server;

#[tokio::main]
async fn main() {
    let server = match Server::new("127.0.0.1:7373").await {
        Ok(v) => v,
        Err(e) => return eprintln!("{e}"),
    };
    println!("Server listening on {}", server.addr);
    tokio::select! {
        _ = server.run() => {}
        _ = tokio::signal::ctrl_c() => {
            eprintln!("Interrupted");
        }
    };
    println!("Server closed");
}
