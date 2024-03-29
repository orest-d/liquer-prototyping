#![allow(unused_imports)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json;

use std::{borrow::Cow, collections::BTreeMap, result::Result};

use liquers_core::error::{Error, ErrorType};
use liquers_core::value::ValueInterface;
use std::convert::{TryFrom, TryInto};

use polars::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ExtValue {
    None,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    Text(String),
    Array(Vec<ExtValue>),
    Object(BTreeMap<String, ExtValue>),
    Bytes(Vec<u8>),
    DataFrame(DataFrame),
}

impl ValueInterface for ExtValue {
    fn none() -> Self {
        ExtValue::None
    }
    fn is_none(&self) -> bool {
        if let ExtValue::None = self {
            true
        } else {
            false
        }
    }

    fn new(txt: &str) -> Self {
        ExtValue::Text(txt.to_owned())
    }

    fn try_into_string(&self) -> Result<String, Error> {
        match self {
            ExtValue::I32(n) => Ok(format!("{n}")),
            ExtValue::I64(n) => Ok(format!("{n}")),
            ExtValue::F64(n) => Ok(format!("{n}")),
            ExtValue::Text(t) => Ok(t.to_owned()),
            ExtValue::Bytes(b) => Ok(String::from_utf8_lossy(b).to_string()),
            _ => Err(Error::conversion_error(self.identifier(), "string")),
        }
    }

    fn try_into_i32(&self) -> Result<i32, Error> {
        match self {
            ExtValue::I32(n) => Ok(*n),
            _ => Err(Error::conversion_error(self.identifier(), "i32")),
        }
    }

    fn try_into_json_value(&self) -> Result<serde_json::Value, Error> {
        match self {
            ExtValue::None => Ok(serde_json::Value::Null),
            ExtValue::Bool(b) => Ok(serde_json::Value::Bool(*b)),
            ExtValue::I32(n) => Ok(serde_json::Value::Number(serde_json::Number::from(*n))),
            ExtValue::I64(n) => Ok(serde_json::Value::Number(serde_json::Number::from(*n))),
            ExtValue::F64(n) => Ok(serde_json::Value::Number(
                serde_json::Number::from_f64(*n).unwrap(),
            )),
            ExtValue::Text(t) => Ok(serde_json::Value::String(t.to_owned())),
            ExtValue::Array(a) => {
                let mut v = Vec::new();
                for x in a {
                    v.push(x.try_into_json_value()?);
                }
                Ok(serde_json::Value::Array(v))
            }
            ExtValue::Object(o) => {
                let mut m = serde_json::Map::new();
                for (k, v) in o {
                    m.insert(k.to_owned(), v.try_into_json_value()?);
                }
                Ok(serde_json::Value::Object(m))
            }
            _ => Err(Error::conversion_error(self.identifier(), "JSON value")),
        }
    }

    fn identifier(&self) -> Cow<'static, str> {
        match self {
            ExtValue::None => "generic".into(),
            ExtValue::Bool(_) => "generic".into(),
            ExtValue::I32(_) => "generic".into(),
            ExtValue::I64(_) => "generic".into(),
            ExtValue::F64(_) => "generic".into(),
            ExtValue::Text(_) => "text".into(),
            ExtValue::Array(_) => "generic".into(),
            ExtValue::Object(_) => "dictionary".into(),
            ExtValue::Bytes(_) => "bytes".into(),
            ExtValue::DataFrame(_) => "polars_dataframe".into(),
        }
    }

    fn type_name(&self) -> Cow<'static, str> {
        match self {
            ExtValue::None => "none".into(),
            ExtValue::Bool(_) => "bool".into(),
            ExtValue::I32(_) => "i32".into(),
            ExtValue::I64(_) => "i64".into(),
            ExtValue::F64(_) => "f64".into(),
            ExtValue::Text(_) => "text".into(),
            ExtValue::Array(_) => "array".into(),
            ExtValue::Object(_) => "object".into(),
            ExtValue::Bytes(_) => "bytes".into(),
            ExtValue::DataFrame(_) => "polars::DataFrame".into(),
        }
    }

    fn default_extension(&self) -> Cow<'static, str> {
        match self {
            ExtValue::None => "json".into(),
            ExtValue::Bool(_) => "json".into(),
            ExtValue::I32(_) => "json".into(),
            ExtValue::I64(_) => "json".into(),
            ExtValue::F64(_) => "json".into(),
            ExtValue::Text(_) => "txt".into(),
            ExtValue::Array(_) => "json".into(),
            ExtValue::Object(_) => "json".into(),
            ExtValue::Bytes(_) => "b".into(),
            ExtValue::DataFrame(_) => "csv".into(),
        }
    }

    fn default_filename(&self) -> Cow<'static, str> {
        match self {
            ExtValue::None => "data.json".into(),
            ExtValue::Bool(_) => "data.json".into(),
            ExtValue::I32(_) => "data.json".into(),
            ExtValue::I64(_) => "data.json".into(),
            ExtValue::F64(_) => "data.json".into(),
            ExtValue::Text(_) => "text.txt".into(),
            ExtValue::Array(_) => "data.json".into(),
            ExtValue::Object(_) => "data.json".into(),
            ExtValue::Bytes(_) => "binary.b".into(),
            ExtValue::DataFrame(_) => "data.csv".into(),
        }
    }

    fn default_media_type(&self) -> Cow<'static, str> {
        match self {
            ExtValue::None => "application/json".into(),
            ExtValue::Bool(_) => "application/json".into(),
            ExtValue::I32(_) => "application/json".into(),
            ExtValue::I64(_) => "application/json".into(),
            ExtValue::F64(_) => "application/json".into(),
            ExtValue::Text(_) => "text/plain".into(),
            ExtValue::Array(_) => "application/json".into(),
            ExtValue::Object(_) => "application/json".into(),
            ExtValue::Bytes(_) => "application/octet-stream".into(),
            ExtValue::DataFrame(_) => "polars_dataframe".into(),
        }
    }

    fn from_string(txt: String) -> Self {
        ExtValue::Text(txt)
    }

    fn from_i32(n: i32) -> Self {
        ExtValue::I32(n)
    }

    fn from_i64(n: i64) -> Self {
        ExtValue::I64(n)
    }

    fn from_f64(n: f64) -> Self {
        ExtValue::F64(n)
    }

    fn from_bool(b: bool) -> Self {
        ExtValue::Bool(b)
    }

    fn from_bytes(b: Vec<u8>) -> Self {
        ExtValue::Bytes(b)
    }

    fn try_from_json_value(value: &serde_json::Value) -> Result<Self, Error> {
        match value {
            serde_json::Value::Null => Ok(ExtValue::None),
            serde_json::Value::Bool(b) => Ok(ExtValue::Bool(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(ExtValue::I64(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(ExtValue::F64(f))
                } else {
                    Err(Error::conversion_error_with_message(
                        value,
                        "i64 or f64",
                        "Invalid JSON number",
                    ))
                }
            }
            serde_json::Value::String(s) => Ok(ExtValue::Text(s.to_owned())),
            serde_json::Value::Array(a) => {
                let mut v = Vec::new();
                for x in a {
                    v.push(ExtValue::try_from_json_value(x)?);
                }
                Ok(ExtValue::Array(v))
            }
            serde_json::Value::Object(o) => {
                let mut m = BTreeMap::new();
                for (k, v) in o {
                    m.insert(k.to_owned(), ExtValue::try_from_json_value(v)?);
                }
                Ok(ExtValue::Object(m))
            }
        }
    }
}

impl TryFrom<&ExtValue> for i32 {
    type Error = Error;
    fn try_from(value: &ExtValue) -> Result<Self, Self::Error> {
        match value {
            ExtValue::I32(x) => Ok(*x),
            ExtValue::I64(x) => i32::try_from(*x)
                .map_err(|e| Error::conversion_error_with_message("I64", "i32", &e.to_string())),
            _ => Err(Error::conversion_error(value.type_name(), "i32")),
        }
    }
}

impl TryFrom<ExtValue> for i32 {
    type Error = Error;
    fn try_from(value: ExtValue) -> Result<Self, Self::Error> {
        match value {
            ExtValue::I32(x) => Ok(x),
            ExtValue::I64(x) => i32::try_from(x)
                .map_err(|e| Error::conversion_error_with_message("I64", "i32", &e.to_string())),
            _ => Err(Error::conversion_error(value.type_name(), "i32")),
        }
    }
}

impl From<i32> for ExtValue {
    fn from(value: i32) -> ExtValue {
        ExtValue::I32(value)
    }
}

impl From<()> for ExtValue {
    fn from(_value: ()) -> ExtValue {
        ExtValue::none()
    }
}

impl TryFrom<ExtValue> for i64 {
    type Error = Error;
    fn try_from(value: ExtValue) -> Result<Self, Self::Error> {
        match value {
            ExtValue::I32(x) => Ok(x as i64),
            ExtValue::I64(x) => Ok(x),
            _ => Err(Error::conversion_error(value.type_name(), "i64")),
        }
    }
}
impl From<i64> for ExtValue {
    fn from(value: i64) -> ExtValue {
        ExtValue::I64(value)
    }
}

impl TryFrom<ExtValue> for f64 {
    type Error = Error;
    fn try_from(value: ExtValue) -> Result<Self, Self::Error> {
        match value {
            ExtValue::I32(x) => Ok(x as f64),
            ExtValue::I64(x) => Ok(x as f64),
            ExtValue::F64(x) => Ok(x),
            _ => Err(Error::conversion_error(value.type_name(), "f64")),
        }
    }
}
impl From<f64> for ExtValue {
    fn from(value: f64) -> ExtValue {
        ExtValue::F64(value)
    }
}

impl TryFrom<ExtValue> for bool {
    type Error = Error;
    fn try_from(value: ExtValue) -> Result<Self, Self::Error> {
        match value {
            ExtValue::I32(x) => Ok(x != 0),
            ExtValue::I64(x) => Ok(x != 0),
            _ => Err(Error::conversion_error(value.type_name(), "bool")),
        }
    }
}
impl From<bool> for ExtValue {
    fn from(value: bool) -> ExtValue {
        ExtValue::Bool(value)
    }
}

impl TryFrom<ExtValue> for String {
    type Error = Error;
    fn try_from(value: ExtValue) -> Result<Self, Self::Error> {
        match value {
            ExtValue::Text(x) => Ok(x),
            ExtValue::I32(x) => Ok(format!("{}", x)),
            ExtValue::I64(x) => Ok(format!("{}", x)),
            ExtValue::F64(x) => Ok(format!("{}", x)),
            _ => Err(Error::conversion_error(value.type_name(), "string")),
        }
    }
}

impl From<String> for ExtValue {
    fn from(value: String) -> ExtValue {
        ExtValue::Text(value)
    }
}
impl From<&str> for ExtValue {
    fn from(value: &str) -> ExtValue {
        ExtValue::Text(value.to_owned())
    }
}

impl liquers_core::value::DefaultValueSerializer for ExtValue {
    fn as_bytes(&self, format: &str) -> Result<Vec<u8>, Error> {
        match format {
            "json" => Err(Error::new(
                ErrorType::SerializationError,
                format!("JSON error"),
            )),
            "txt" | "html" => match self {
                ExtValue::None => Ok("none".as_bytes().to_vec()),
                ExtValue::Bool(true) => Ok("true".as_bytes().to_vec()),
                ExtValue::Bool(false) => Ok("false".as_bytes().to_vec()),
                ExtValue::I32(x) => Ok(format!("{x}").into_bytes()),
                ExtValue::I64(x) => Ok(format!("{x}").into_bytes()),
                ExtValue::F64(x) => Ok(format!("{x}").into_bytes()),
                ExtValue::Text(x) => Ok(x.as_bytes().to_vec()),
                _ => Err(Error::new(
                    ErrorType::SerializationError,
                    format!(
                        "Serialization to {} not supported by {}",
                        format,
                        self.type_name()
                    ),
                )),
            },
            _ => Err(Error::new(
                ErrorType::SerializationError,
                format!("Unsupported format {}", format),
            )),
        }
    }
    fn deserialize_from_bytes(b: &[u8], _type_identifier: &str, fmt: &str) -> Result<Self, Error> {
        match fmt {
            "json" => Err(Error::new(
                ErrorType::SerializationError,
                format!("JSON error in from_bytes"),
            )),
            _ => Err(Error::new(
                ErrorType::SerializationError,
                format!("Unsupported format in from_bytes:{}", fmt),
            )),
        }
    }
}

impl From<DataFrame> for ExtValue {
    fn from(value: DataFrame) -> ExtValue {
        ExtValue::DataFrame(value)
    }
}
