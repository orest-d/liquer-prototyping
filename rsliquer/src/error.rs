use std::error;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Error{
    ArgumentNotSpecified,
    ParameterError{message:String},
    ConversionError{message:String},
    SerializationError{message:String, format:String},
    General{message:String}
}

impl fmt::Display for Error{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ArgumentNotSpecified => write!(f, "Argument not specified"),
            Error::ParameterError{message} => write!(f, "Error: {}", message),
            Error::ConversionError{message} => write!(f, "Error: {}", message),
            Error::SerializationError{message, format:_} => write!(f, "Error: {}", message),
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
