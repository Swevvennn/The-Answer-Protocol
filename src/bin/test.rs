use tap::messages::{Message, Error};

fn test(str: &str) {
    println!("===== {} =====", str);
    let message = match Message::from_string(str) {
        Ok(v) => v,
        Err(e) => {
            println!("INVALID: {}", e);
            return;
        }
    };
    match message {
        Message::Command(v) => println!("Command: {}", v),
        Message::Error(v) => println!("Error: {}", v),
        Message::Event(v) => println!("Event: {}", v),
        Message::Response(v) => println!("Response: {}", v),
    }
}

fn main() {
    test("abc");
    test("CHAT abc");
    test(&Error::AlreadyInGroup.to_string());
    test("ERR 401 ALREADY_IN_GROUP");
    test("OK");
    test(" OK");
    test("OK ");
    test("OK abc def");
}
