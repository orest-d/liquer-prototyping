use serde_json;

use std::{result::Result, collections::BTreeMap};

use crate::error::Error;
use std::convert::{TryFrom, TryInto};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value{
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

trait ValueType{
    /// String identifier of the state type
    fn identifier(&self)->&'static str;

    /// Default file extension; determines the default data format
    /// Must be consistent with the default_mimetype.
    fn default_extension(&self)->&'static str;

    /// Default file name
    fn default_filename(&self)->&'static str;

    /// Default mime type - must be consistent with the default_extension
    fn default_mimetype(&self)->&'static str;
}

impl ValueType for Value{
    fn identifier(&self)->&'static str{
        match self{
            Value::None => "none",
            Value::Bool(_) => "bool",
            Value::I32(_) => "i32",
            Value::I64(_) => "i64",
            Value::F64(_) => "f64",
            Value::Text(_) => "text",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Bytes(_) => "bytes",
        }
    }

    fn default_extension(&self)->&'static str{
        match self{
            Value::None => "json",
            Value::Bool(_) => "json",
            Value::I32(_) => "json",
            Value::I64(_) => "json",
            Value::F64(_) => "json",
            Value::Text(_) => "txt",
            Value::Array(_) => "json",
            Value::Object(_) => "json",
            Value::Bytes(_) => "b",
        }
    }

    fn default_filename(&self)->&'static str{
        match self{
            Value::None => "data.json",
            Value::Bool(_) => "data.json",
            Value::I32(_) => "data.json",
            Value::I64(_) => "data.json",
            Value::F64(_) => "data.json",
            Value::Text(_) => "text.txt",
            Value::Array(_) => "data.json",
            Value::Object(_) => "data.json",
            Value::Bytes(_) => "binary.b",
        }
    }

    fn default_mimetype(&self)->&'static str{
        match self{
            Value::None => "application/json",
            Value::Bool(_) => "application/json",
            Value::I32(_) => "application/json",
            Value::I64(_) => "application/json",
            Value::F64(_) => "application/json",
            Value::Text(_) => "text/plain",
            Value::Array(_) => "application/json",
            Value::Object(_) => "application/json",
            Value::Bytes(_) => "application/octet-stream",
        }
    }
}

impl TryFrom<Value> for i32{
    type Error=Error;
    fn try_from(value: Value) -> Result<Self, Self::Error>{
        match value{
            Value::I32(x) => Ok(x),
            Value::I64(x) => i32::try_from(x).map_err(|e| Error::ConversionError{message:format!("I64 to i32 conversion error {e}")}),
            _ => Err(Error::ConversionError{message:format!("Can't convert {} to i32", value.identifier())}),
        }
    }
}

impl From<i32> for Value{
    fn from(value: i32) -> Value{
        Value::I32(value)
    }
}

impl TryFrom<Value> for String{
    type Error=Error;
    fn try_from(value: Value) -> Result<Self, Self::Error>{
        match value{
            Value::Text(x) => Ok(x),
            Value::I32(x) => Ok(format!("{}",x)),
            Value::I64(x) => Ok(format!("{}",x)),
            Value::F64(x) => Ok(format!("{}",x)),           
            _ => Err(Error::ConversionError{message:format!("Can't convert {} to string", value.identifier())}),
        }
    }
}

impl From<String> for Value{
    fn from(value: String) -> Value{
        Value::Text(value)
    }
}
impl From<&str> for Value{
    fn from(value: &str) -> Value{
        Value::Text(value.to_owned())
    }
}


trait ValueSerializer where Self:Sized{
    fn as_bytes(&self, format:&str)->Result<Vec<u8>, Error>;
    fn from_bytes(b: &[u8], format:&str)->Result<Self, Error>;
}

impl ValueSerializer for Value{
    fn as_bytes(&self, format:&str)->Result<Vec<u8>, Error>{
        match format{
            "json" => serde_json::to_vec(self).map_err(|e| Error::SerializationError{message:format!("JSON errror {}",e), format:format.to_owned()}),
            _ => Err(Error::SerializationError{message:format!("Unsupported format {}",format), format:format.to_owned()})
        }
    }
    fn from_bytes(b: &[u8], format:&str)->Result<Self, Error>{
        match format{
            "json" => serde_json::from_slice(b).map_err(|e| Error::SerializationError{message:format!("JSON errror {}",e), format:format.to_owned()}),
            _ => Err(Error::SerializationError{message:format!("Unsupported format {}",format), format:format.to_owned()})
        }
    }
}


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test1() -> Result<(), Box<dyn std::error::Error>>{
        println!("Hello.");
        let v = Value::I32(123);
        let b = v.as_bytes("json")?;
        println!("Serialized    {:?}: {}", v, std::str::from_utf8(&b)?);
        let w:Value = ValueSerializer::from_bytes(&b, "json")?;
        println!("De-Serialized {:?}", w);
        Ok(())
    }   
    #[test]
    fn test_convert_int() -> Result<(), Box<dyn std::error::Error>>{
        let v = Value::I32(123);
        let x:i32 = v.try_into()?;
        assert_eq!(x,123);
        Ok(())
    }   
    #[test]
    fn test_convert_text() -> Result<(), Box<dyn std::error::Error>>{
        let v = Value::from("abc");
        assert_eq!(v,Value::Text("abc".to_owned()));
        let x:String = v.try_into()?;
        assert_eq!(x,"abc");
        Ok(())
    }   
}