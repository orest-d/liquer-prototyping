use crate::query::Position;
use std::error;
use std::fmt;
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Error {
    ArgumentNotSpecified,
    ActionNotRegistered { message: String },
    CommandAlreadyRegistered { message: String },
    ParseError { message: String, position: Position },
    ParameterError { message: String, position: Position },
    ConversionError { message: String, position: Position },
    SerializationError { message: String, format: String },
    General { message: String },
    CacheNotSupported,
    NotSupported { message: String },
}

impl Error{
    pub fn missing_argument(i:usize, name:&str, position:&Position) -> Self{
        Error::ParameterError{
            message: format!("Missing argument #{}:{}", i, name),
            position: position.clone(),
        }
    }
    pub fn conversion_error<W:Display,T:Display>(what:W,to:T) -> Self {
        Error::ConversionError {
            message: format!("Can't convert '{}' to {}", what, to),
            position: Position::unknown()
        }
    }
    pub fn conversion_error_at_position<W:Display,T:Display>(what:W,to:T,position:&Position) -> Self {
        Error::ConversionError {
            message: format!("Can't convert '{}' to {}", what, to),
            position: position.clone()
        }
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ArgumentNotSpecified => write!(f, "Argument not specified"),
            Error::ActionNotRegistered { message } => write!(f, "Error: {}", message),
            Error::ParseError { message, position } => write!(f, "Error: {} {}", message, position),
            Error::ParameterError { message, position } => {
                write!(f, "Error: {} {}", message, position)
            },
            Error::ConversionError { message, position } => {
                if position.is_unknown() {
                    write!(f, "Error: {}", message)
                } else {
                    write!(f, "Error: {} at {}", message, position)
                }
            },
            Error::SerializationError { message, format: _ } => write!(f, "Error: {}", message),
            Error::General { message } => write!(f, "Error: {}", message),
            Error::CacheNotSupported => write!(f, "Error: Cache not supported"),
            Error::NotSupported { message } => write!(f, "Error: {}", message),
            Error::CommandAlreadyRegistered { message } => write!(f, "Error: {}", message),
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
