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
        payload: messages::Payload::new(Test {
            s: "String".to_string(),
            i: 73,
        })
    };
    println!("{}", event);
    let test: Test = event.payload.extract();
    print!("{}", test.s);
}