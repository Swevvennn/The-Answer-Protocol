use std::str::FromStr;

use tap::messages::{Message, PayloadExtractor, PayloadKind};

fn test(str: &str) {
    println!("===== TEST: {} =====", str);
    let message = match Message::from_str(str) {
        Ok(v) => v,
        Err(e) => {
            println!("INVALID: {}", e);
            return;
        }
    };
    let payload = match &message {
        Message::Command(v) => {println!("Command: {}", v); &v.payload},
        Message::Error(v) => {println!("Error: {}", v); return},
        Message::Event(v) => {println!("Event: {}", v); &v.payload},
        Message::Response(v) => {println!("Response: {}", v); &v.payload},
    };
    for arg in &payload.args {
        match arg {
            PayloadKind::String(v) => println!("Arg<String>: {}", v),
            PayloadKind::KeyValue { key, value } => println!("Arg<KeyValue>: {}={}", key, value),
            PayloadKind::Json(v) => println!("Arg<Json>: {}", v),
        }
    }
}

// fn extract(str: &str) {
//     println!("===== EXTRACT: {} =====", str);
//     let message = match Message::from_str(str) {
//         Ok(v) => v,
//         Err(e) => {
//             println!("INVALID: {}", e);
//             return;
//         }
//     };
//     let payload = match &message {
//         Message::Command(v) => {println!("Command: {}", v); &v.payload},
//         Message::Error(v) => {println!("Error: {}", v); return},
//         Message::Event(v) => {println!("Event: {}", v); &v.payload},
//         Message::Response(v) => {println!("Response: {}", v); &v.payload},
//     };
//     #[derive(Debug, serde::Deserialize)]
//     #[serde(deny_unknown_fields)]
//     struct Abc {
//         i: u32,
//         s: String,
//     }
//     let mut s = "hello".to_string();
//     let mut k = "key".to_string();
//     let mut v = String::new();
//     let mut abc = Abc {
//         i: 0,
//         s: String::new(),
//     };
//     if let Err(e) = payload.extract(&mut [
//         PayloadExtractor::String(&mut s),
//         PayloadExtractor::KeyValue { key: &mut k, value: &mut v },
//         PayloadExtractor::Json(&mut abc),
//     ]) {
//         println!("INVALID: {e}");
//         return;
//     }
//     println!("s = {s}");
//     println!("k = {k}");
//     println!("v = {v}");
//     println!("abc = {:?}", abc);
// }

// fn test(i: i32) -> Result<i32, std::io::Error> {
//     if i == 0 {
//         return Err(std::io::Error::other("error error error"));
//     }
//     i
// }

fn main() {
    // extract("OK hello keyd=value { \"i\": 2147483647, \"s\": \"string\" }");
    // test("abc");
    // test("CHAT ab\\ c");
    // test(&Error::AlreadyInGroup.to_string());
    // test("ERR 401 ALREADY_IN_GROUP");
    test("OK");
    test(" OK");
    test("OK ");
    test("LOOK     ");
    test("MOVE  north");
    test("OK abc def=ghi {\"a\": 73, \"b\": [4, 5, 6]}");

    // let t = 0;
    // match match t {
    //     0 => {
    //         println!("i: {}", test(0)?);
    //         Ok(())
    //     }
    //     _ => {
    //         println!("i: {}", t);
    //         Ok(())
    //     }
    // } {
    //     Ok(v) => println!("Ok: {v}"),
    //     Err(e) => println!("Error: {e}"),
    // }
}


// async fn run_client() {}

// #[tokio::main]
// async fn main() {
//     let mut server = tap::network::Server::default();
//     server.addr = "127.0.0.1:7373".to_string();
//     server.bind().await.unwrap();

//     let mut clients: Vec<std::sync::Arc<tap::network::Client>> = vec![];

//     loop {
//         let client = std::sync::Arc::new(server.accept().await.unwrap());
//         clients.push(client.clone());
//         tokio::spawn(async move {
//             loop {
//                 let message = client.read().await.unwrap();
//             }
//         });
//     }
// }