use serde::{Deserialize, Serialize};
use tap::messages;

#[derive(Serialize, Deserialize)]
struct Test {
    pub s: String,
    pub i: u16,
}

fn main() {
    let event = messages::Event {
        scope: messages::EventScope::Global,
        kind: messages::EventKind::Chat,
        payload: messages::Payload { args: vec![
            messages::PayloadKind::new(Test {
                s: "String".to_string(),
                i: 73,
            }),
            messages::PayloadKind::String("abc".to_string()),
            messages::PayloadKind::KeyValue { key: "key".to_string(), value: "value".to_string() }
        ] }
    };
    println!("{}", event);
    let test: Test = event.payload.args[0].extract();
    print!("{}", test.s);
}