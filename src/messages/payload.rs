use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt;
use std::io::Error;

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
    pub fn new<T: Serialize>(data: T) -> Self {
        Self::Json(serde_json::to_value(data).unwrap())
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn is_key_value(&self) -> bool {
        matches!(self, Self::KeyValue { .. })
    }

    pub fn is_json(&self) -> bool {
        matches!(self, Self::Json(_))
    }

    pub fn extract<T: DeserializeOwned>(&self) -> Result<T, Error> {
        match self {
            Self::Json(json) => serde_json::from_value(json.clone()).map_err(|e| e.into()),
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
    pub fn from_string(str: String) -> Result<Payload, Error> {
        Ok(Payload { args: vec![ PayloadKind::String(str) ] })
    }
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
