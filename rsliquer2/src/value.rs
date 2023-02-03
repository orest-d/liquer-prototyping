use serde_json;

use std::{borrow::Cow, collections::BTreeMap, result::Result};

use crate::error::Error;
use std::convert::{TryFrom, TryInto};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Value {
    None,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    Text(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Bytes(Vec<u8>),
}

pub trait ValueInterface {
    /// Empty value
    fn none() -> Self;

    /// From string
    fn new(txt: &str) -> Self;

    /// String identifier of the state type
    /// Several types can be linked to the same identifier.
    /// The identifier must be cross-platform
    fn identifier(&self) -> Cow<'static, str>;

    /// String name of the stored type
    /// The type_name is more detailed than identifier.
    /// The identifier does not need to be cross-platform, it serves more for information and debugging
    fn type_name(&self) -> Cow<'static, str>;

    /// Default file extension; determines the default data format
    /// Must be consistent with the default_mimetype.
    fn default_extension(&self) -> Cow<'static, str>;

    /// Default file name
    fn default_filename(&self) -> Cow<'static, str>;

    /// Default mime type - must be consistent with the default_extension
    fn default_mimetype(&self) -> Cow<'static, str>;
}

impl ValueInterface for Value {
    fn none() -> Self {
        Value::None
    }

    fn new(txt: &str) -> Self {
        Value::Text(txt.to_owned())
    }
    fn identifier(&self) -> Cow<'static, str> {
        match self {
            Value::None => "none".into(),
            Value::Bool(_) => "generic".into(),
            Value::I32(_) => "generic".into(),
            Value::I64(_) => "generic".into(),
            Value::F64(_) => "generic".into(),
            Value::Text(_) => "text".into(),
            Value::Array(_) => "generic".into(),
            Value::Object(_) => "dictionary".into(),
            Value::Bytes(_) => "bytes".into(),
        }
    }

    fn type_name(&self) -> Cow<'static, str> {
        match self {
            Value::None => "none".into(),
            Value::Bool(_) => "bool".into(),
            Value::I32(_) => "i32".into(),
            Value::I64(_) => "i64".into(),
            Value::F64(_) => "f64".into(),
            Value::Text(_) => "text".into(),
            Value::Array(_) => "array".into(),
            Value::Object(_) => "object".into(),
            Value::Bytes(_) => "bytes".into(),
        }
    }

    fn default_extension(&self) -> Cow<'static, str> {
        match self {
            Value::None => "json".into(),
            Value::Bool(_) => "json".into(),
            Value::I32(_) => "json".into(),
            Value::I64(_) => "json".into(),
            Value::F64(_) => "json".into(),
            Value::Text(_) => "txt".into(),
            Value::Array(_) => "json".into(),
            Value::Object(_) => "json".into(),
            Value::Bytes(_) => "b".into(),
        }
    }

    fn default_filename(&self) -> Cow<'static, str> {
        match self {
            Value::None => "data.json".into(),
            Value::Bool(_) => "data.json".into(),
            Value::I32(_) => "data.json".into(),
            Value::I64(_) => "data.json".into(),
            Value::F64(_) => "data.json".into(),
            Value::Text(_) => "text.txt".into(),
            Value::Array(_) => "data.json".into(),
            Value::Object(_) => "data.json".into(),
            Value::Bytes(_) => "binary.b".into(),
        }
    }

    fn default_mimetype(&self) -> Cow<'static, str> {
        match self {
            Value::None => "application/json".into(),
            Value::Bool(_) => "application/json".into(),
            Value::I32(_) => "application/json".into(),
            Value::I64(_) => "application/json".into(),
            Value::F64(_) => "application/json".into(),
            Value::Text(_) => "text/plain".into(),
            Value::Array(_) => "application/json".into(),
            Value::Object(_) => "application/json".into(),
            Value::Bytes(_) => "application/octet-stream".into(),
        }
    }
}

impl TryFrom<Value> for i32 {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::I32(x) => Ok(x),
            Value::I64(x) => i32::try_from(x).map_err(|e| Error::ConversionError {
                message: format!("I64 to i32 conversion error {e}"),
            }),
            _ => Err(Error::ConversionError {
                message: format!("Can't convert {} to i32", value.type_name()),
            }),
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Value {
        Value::I32(value)
    }
}

impl TryFrom<Value> for String {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Text(x) => Ok(x),
            Value::I32(x) => Ok(format!("{}", x)),
            Value::I64(x) => Ok(format!("{}", x)),
            Value::F64(x) => Ok(format!("{}", x)),
            _ => Err(Error::ConversionError {
                message: format!("Can't convert {} to string", value.type_name()),
            }),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Value {
        Value::Text(value)
    }
}
impl From<&str> for Value {
    fn from(value: &str) -> Value {
        Value::Text(value.to_owned())
    }
}

trait ValueSerializer
where
    Self: Sized,
{
    fn as_bytes(&self, format: &str) -> Result<Vec<u8>, Error>;
    fn from_bytes(b: &[u8], format: &str) -> Result<Self, Error>;
}

impl ValueSerializer for Value {
    fn as_bytes(&self, format: &str) -> Result<Vec<u8>, Error> {
        match format {
            "json" => serde_json::to_vec(self).map_err(|e| Error::SerializationError {
                message: format!("JSON error {}", e),
                format: format.to_owned(),
            }),
            _ => Err(Error::SerializationError {
                message: format!("Unsupported format {}", format),
                format: format.to_owned(),
            }),
        }
    }
    fn from_bytes(b: &[u8], format: &str) -> Result<Self, Error> {
        match format {
            "json" => serde_json::from_slice(b).map_err(|e| Error::SerializationError {
                message: format!("JSON error {}", e),
                format: format.to_owned(),
            }),
            _ => Err(Error::SerializationError {
                message: format!("Unsupported format {}", format),
                format: format.to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() -> Result<(), Box<dyn std::error::Error>> {
        println!("Hello.");
        let v = Value::I32(123);
        let b = v.as_bytes("json")?;
        println!("Serialized    {:?}: {}", v, std::str::from_utf8(&b)?);
        let w: Value = ValueSerializer::from_bytes(&b, "json")?;
        println!("De-Serialized {:?}", w);
        Ok(())
    }
    #[test]
    fn test_convert_int() -> Result<(), Box<dyn std::error::Error>> {
        let v = Value::I32(123);
        let x: i32 = v.try_into()?;
        assert_eq!(x, 123);
        Ok(())
    }
    #[test]
    fn test_convert_text() -> Result<(), Box<dyn std::error::Error>> {
        let v = Value::from("abc");
        assert_eq!(v, Value::Text("abc".to_owned()));
        let x: String = v.try_into()?;
        assert_eq!(x, "abc");
        Ok(())
    }
    #[test]
    fn test_serde_to_json() -> Result<(), Box<dyn std::error::Error>> {
        let v = Value::None;
        let s = serde_json::to_string(&v)?;
        assert_eq!(s, "null");
        let v = Value::Bool(true);
        let s = serde_json::to_string(&v)?;
        assert_eq!(s, "true");
        let v = Value::I32(123);
        let s = serde_json::to_string(&v)?;
        assert_eq!(s, "123");
        let v = Value::I64(123);
        let s = serde_json::to_string(&v)?;
        assert_eq!(s, "123");
        let v = Value::F64(123.456);
        let s = serde_json::to_string(&v)?;
        assert_eq!(s, "123.456");
        let v = Value::from("abc");
        let s = serde_json::to_string(&v)?;
        assert_eq!(s, "\"abc\"");
        let v = Value::Array(vec![Value::None, Value::Bool(false), Value::I32(123)]);
        let s = serde_json::to_string(&v)?;
        assert_eq!(s, "[null,false,123]");
        let mut m = BTreeMap::new();
        m.insert("test".to_owned(), Value::None);
        m.insert("a".to_owned(), Value::I32(123));
        let v = Value::Object(m);
        let s = serde_json::to_string(&v)?;
        assert_eq!(s, "{\"a\":123,\"test\":null}");
        Ok(())
    }
    #[test]
    fn test_serde_from_json() -> Result<(), Box<dyn std::error::Error>> {
        let v: Value = serde_json::from_str("null")?;
        assert_eq!(v, Value::None);
        let v: Value = serde_json::from_str("true")?;
        assert_eq!(v, Value::Bool(true));
        let v: Value = serde_json::from_str("123")?;
        assert_eq!(v, Value::I32(123));
        let v: Value = serde_json::from_str("123456789123456789")?;
        assert_eq!(v, Value::I64(123456789123456789));
        let v: Value = serde_json::from_str("123.456")?;
        assert_eq!(v, Value::F64(123.456));
        let v: Value = serde_json::from_str("[null, true, 123]")?;
        assert_eq!(
            v,
            Value::Array(vec![Value::None, Value::Bool(true), Value::I32(123)])
        );
        let v: Value = serde_json::from_str("{\"a\":123,\"test\":null}")?;
        if let Value::Object(x) = v {
            assert_eq!(x.len(), 2);
            assert_eq!(x["a"], Value::I32(123));
            assert_eq!(x["test"], Value::None);
        } else {
            assert!(false);
        }
        Ok(())
    }
}
