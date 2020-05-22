use serde;
use serde_json;
use serde_yaml;

#[macro_use]
use serde_derive;

use std::collections::HashMap;

use std::result::Result;
use std::error;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Error{
    SerializationError{message:String, format:String},
    General{message:String}
}

impl fmt::Display for Error{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SerializationError{message, format} => write!(f, "Error: {}", message),
            Error::General{message} => write!(f, "Error: {}", message),
        }
    }    
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            _ => None,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
enum Value{
    None,
    Text(String),
    Integer(i32),
    Real(f64)
}

trait ValueSerializer where Self:Sized{
    fn identifier(&self)->String;
    fn default_extension(&self)->String;
    fn default_media_type(&self)->String;
    fn as_bytes(&self, format:&str)->Result<Vec<u8>, Error>;
    fn from_bytes(b: &[u8], format:&str)->Result<Self, Error>;
}

impl ValueSerializer for Value{
    fn identifier(&self)->String{
        match self {
            Value::None => String::from("none"),
            Value::Text(_) => String::from("text"),
            Value::Integer(_) => String::from("int"),
            Value::Real(_) => String::from("real"),
        }
    }
    fn default_extension(&self)->String{
        String::from("json")
    }
    fn default_media_type(&self)->String{
        String::from("application/json")
    }
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
        let v = Value::Integer(123);
        let b = v.as_bytes("json")?;
        println!("Serialized    {:?}: {}", v, std::str::from_utf8(&b)?);
        let w:Value = ValueSerializer::from_bytes(&b, "json")?;
        println!("De-Serialized {:?}", w);
        Ok(())
    }   
}