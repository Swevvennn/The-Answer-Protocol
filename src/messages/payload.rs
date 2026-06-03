use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt;

use crate::messages::utils::write_vec;

pub enum PayloadKind {
    String(String),
    KeyValue {
        key: String,
        value: String
    },
    Json(Value),
}

impl PayloadKind {
    pub fn new<T: Serialize>(json: T) -> Self {
        Self::Json(serde_json::to_value(json).unwrap())
    }

    pub fn extract<T: DeserializeOwned>(&self) -> T {
        match self {
            Self::Json(json) => serde_json::from_value(json.clone()).unwrap(),
            _ => panic!("payload isn't a json"),
        }
    }
}

impl fmt::Display for PayloadKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(str) => write!(f, "{}", str),
            Self::KeyValue { key, value } => write!(f, "{}={}", key, value),
            Self::Json(json) => write!(f, "{}", serde_json::to_string(&json).map_err(|_| fmt::Error)?),
        }
    }
}

pub struct Payload {
    pub args: Vec<PayloadKind>,
}

impl Payload {
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut v = vec![];
        for arg in &self.args {
            v.push(arg.to_string());
        }
        write_vec(f, v)
    }
}

// pub struct Payload {
//     data: Value,
// }

// impl Payload {
//     pub fn new<T: Serialize>(data: T) -> Self {
//         Self {
//             data: serde_json::to_value(data).unwrap(),
//         }
//     }

//     pub fn extract<T: DeserializeOwned>(&self) -> T {
//         serde_json::from_value(self.data.clone()).unwrap()
//     }
// }

// impl fmt::Display for Payload {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "{}",
//             serde_json::to_string(&self.data)
//                 .map_err(|_| fmt::Error)?
//         )
//     }
// }
