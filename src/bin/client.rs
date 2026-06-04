use std::thread::sleep;
use std::time::Duration;

use tap::network::Client;

#[tokio::main]
async fn main() {
    let mut client = match Client::connect("127.0.0.1:7878").await {
        Ok(v) => v,
        Err(e) => return eprintln!("{e}"),
    };
    loop {
        match client.write("CHAT Hello\\ World!\n").await {
            Ok(_) => (),
            Err(e) => return eprintln!("{e}")
        };
        sleep(Duration::from_secs(1));
    }
}
