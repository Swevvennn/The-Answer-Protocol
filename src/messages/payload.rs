use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt;
use std::io::Error;

use crate::messages::MessageParse;
use crate::messages::utils::{invalid_input, write_vec};

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

    pub fn string_from_string(s: String) -> Result<Self, std::io::Error> {
        Ok(Self::String(Self::unescape(&s)))
    }

    pub fn key_value_from_string(s: String) -> Result<Self, std::io::Error> {
        let parts: Vec<&str> = s.split("=").collect();
        if parts.len() != 2 {
            return Err(invalid_input("invalid key value"));
        }
        Ok(Self::KeyValue {
            key: Self::unescape(parts[0]),
            value: Self::unescape(parts[1])
        })
    }

    pub fn json_from_string(s: String) -> Result<Self, std::io::Error> {
        match serde_json::from_str(&s) {
            Ok(v) => Ok(Self::Json(v)),
            Err(_) => Err(invalid_input("invalid json")),
        }
    }

    pub fn extract<T: DeserializeOwned>(&self) -> Result<T, Error> {
        match self {
            Self::Json(json) => serde_json::from_value(json.clone()).map_err(|e| e.into()),
            _ => panic!("payload isn't a json"),
        }
    }

    fn escape(s: &str) -> String {
        let mut res = String::new();
        for char in s.chars() {
            if matches!(char, '\\' | ' ' | '=' | '{' | '}' | '[' | ']') {
                res.push('\\');
            }
            res.push(char);
        }
        res
    }

    fn unescape(s: &str) -> String {
        let mut res = String::new();
        let mut escape = false;
        for char in s.chars() {
            if !escape && char == '\\' {
                escape = true;
            } else {
                escape = false;
                res.push(char);
            }
        }
        res
    }
}

impl fmt::Display for PayloadKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", Self::escape(s)),
            Self::KeyValue { key, value } => write!(f, "{}={}", Self::escape(key), Self::escape(value)),
            Self::Json(json) => write!(f, "{}", serde_json::to_string(&json).map_err(|_| fmt::Error)?),
        }
    }
}

pub struct Payload {
    pub args: Vec<PayloadKind>,
}

impl MessageParse for Payload {
    fn from_string(s: String) -> Result<Self, Error> {
        let mut payload = Self { args: Vec::new() };
        let mut escaped = false;
        let mut i: usize = 0;
        let chars: Vec<char> = s.chars().collect();
        while i < chars.len() {
            let mut j = i;
            let mut t: i8 = 0;
            let mut level = 0;
            let mut in_string = false;
            while j < chars.len() {
                if !escaped {
                    if chars[j] == '\\' && (t != 2 || in_string) {
                        escaped = true;
                    } else if chars[j] == ' ' && (t != 2 || level == 0) {
                        break;
                    } else if chars[j] == '=' {
                        t += 1;
                    } else if matches!(chars[j], '{' | '}' | '[' | ']') {
                        t = 2;
                        if !in_string {
                            if matches!(chars[j], '{' | '[') {
                                level += 1;
                            } else {
                                level -= 1;
                            }
                        }
                    } else if t == 2 && chars[j] == '"' {
                        in_string = !in_string;
                    }
                }
                j += 1;
            }
            let arg = chars[i..j].iter().collect();
            let kind = match t {
                1 => PayloadKind::key_value_from_string(arg),
                2 => PayloadKind::json_from_string(arg),
                _ => PayloadKind::string_from_string(arg),
            };
            payload.args.push(match kind {
                Ok(v) => v,
                Err(_) => return Err(invalid_input("invalid payload")),
            });
            i = j + 1;
        }
        Ok(payload)
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
