use tap::messages::Error;

fn test(str: &str) {
    println!("===== {} =====", str);
    let message = match Error::from_string(str.to_string()) {
        Ok(v) => v,
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };
    println!("Ok: {}", message);
}

fn main() {
    test("abc");
    test("CHAT abc");
    test(&Error::AlreadyInGroup.to_string());
    test("ERR 401 ALREADY_IN_GROUP");
}


// use serde::{Deserialize, Serialize};
// use tap::messages;
// use tokio::net::windows::named_pipe::PipeMode::Message;

// #[derive(Serialize, Deserialize)]
// struct Test {
//     pub s: String,
//     pub i: u16,
// }

// #[derive(Serialize, Deserialize)]
// struct Abc {
//     pub y: String,
//     pub i: u16,
// }

// fn main() {
//     let event = messages::Event {
//         scope: messages::EventScope::Global,
//         kind: messages::EventKind::Chat,
//         payload: messages::Payload { args: vec![
//             messages::PayloadKind::new(Test {
//                 s: "String".to_string(),
//                 i: 73,
//             }),
//             messages::PayloadKind::String("abc".to_string()),
//             messages::PayloadKind::KeyValue { key: "key".to_string(), value: "value".to_string() }
//         ] }
//     };
//     println!("{}", event);
//     let test: Test = event.payload.args[0].extract().unwrap();
//     print!("{}", test.s);

//     {
//         let abc: Abc = match event.payload.args[0].extract() {
//             Ok(abc) => abc,
//             Err(e) => {
//                 println!("{}", e);
//                 return;
//             },
//         };
//     }
// }
