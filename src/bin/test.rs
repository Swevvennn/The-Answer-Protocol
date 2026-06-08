use tap::messages::{Message, PayloadExtractor, PayloadKind};

fn test(str: &str) {
    println!("===== TEST: {} =====", str);
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

fn extract(str: &str) {
    println!("===== EXTRACT: {} =====", str);
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
    #[derive(Debug, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Abc {
        i: u32,
        s: String,
    }
    let mut s = "hello".to_string();
    let mut k = "key".to_string();
    let mut v = String::new();
    let mut abc = Abc {
        i: 0,
        s: String::new(),
    };
    if let Err(e) = payload.extract(&mut [
        PayloadExtractor::String(&mut s),
        PayloadExtractor::KeyValue { key: &mut k, value: &mut v },
        PayloadExtractor::Json(&mut abc),
    ]) {
        println!("INVALID: {e}");
        return;
    }
    println!("s = {s}");
    println!("k = {k}");
    println!("v = {v}");
    println!("abc = {:?}", abc);
}

fn main() {
    extract("OK hello keyd=value { \"i\": 2147483647, \"s\": \"string\" }");
    // test("abc");
    // test("CHAT ab\\ c");
    // test(&Error::AlreadyInGroup.to_string());
    // test("ERR 401 ALREADY_IN_GROUP");
    // test("OK");
    // test(" OK");
    // test("OK ");
    // test("OK abc def=ghi {\"a\": 73, \"b\": [4, 5, 6]}");
}
