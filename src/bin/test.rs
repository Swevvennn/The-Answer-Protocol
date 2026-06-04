use tap::messages::{Message, Error, PayloadKind};

fn test(str: &str) {
    println!("===== {} =====", str);
    let message = match Message::from_string(str) {
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

fn main() {
    test("abc");
    test("CHAT ab\\ c");
    test(&Error::AlreadyInGroup.to_string());
    test("ERR 401 ALREADY_IN_GROUP");
    test("OK");
    test(" OK");
    test("OK ");
    test("OK abc def=ghi {\"a\": 73, \"b\": [4, 5, 6]}");
}
