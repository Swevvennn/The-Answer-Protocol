use tap::network::Client;
use std::thread::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let mut client = Client::connect("127.0.0.1:7878")
        .await;
    loop {
        client
            .write("Test")
            .await;
        sleep(Duration::from_secs(1));
    }
}
