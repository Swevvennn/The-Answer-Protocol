use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::fmt;

pub struct Payload {
    data: Value,
}

impl Payload {
    pub fn new<T: Serialize>(data: T) -> Self {
        Self {
            data: serde_json::to_value(data).unwrap(),
        }
    }

    pub fn extract<T: DeserializeOwned>(&self) -> T {
        serde_json::from_value(self.data.clone()).unwrap()
    }
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self.data)
                .map_err(|_| fmt::Error)?
        )
    }
}
