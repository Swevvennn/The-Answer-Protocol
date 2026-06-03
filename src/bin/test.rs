use serde::{Deserialize, Serialize};
use tap::messages;

#[derive(Serialize, Deserialize)]
struct Test {
    pub s: String,
    pub i: u16,
}

fn main() {
    let event = messages::Event::new(
        messages::EventScope::Global,
        messages::EventKind::Chat,
        Test {
            s: "Caca".to_string(),
            i: 73,
        }
    );
    println!("{}", event);
    let test: Test = event.extract();
    print!("{}", test.s);
}