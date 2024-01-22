use itertools::Itertools;

use crate::query::ActionRequest;
use crate::query::Position;
use std::error;
use std::fmt;
use std::fmt::Display;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ErrorType {
    ArgumentMissing,
    ActionNotRegistered,
    CommandAlreadyRegistered,
    ParseError,
    ParameterError,
    TooManyParameters,
    ConversionError,
    SerializationError,
    General,
    CacheNotSupported,
    NotSupported,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub error_type: ErrorType,
    pub message: String,
    pub position: Position,
    pub query: Option<String>,
}

impl Error {
    pub fn new(error_type: ErrorType, message: String) -> Self {
        Error {
            error_type: error_type,
            message: message,
            position: Position::unknown(),
            query: None,
        }
    }
    pub fn with_position(mut self, position: &Position) -> Self {
        self.position = position.clone();
        self
    }
    pub fn with_query(mut self, query: &crate::query::Query) -> Self {
        self.query = Some(query.encode());
        self
    }
    pub fn cache_not_supported() -> Self {
        Error {
            error_type: ErrorType::CacheNotSupported,
            message: "Cache not supported".to_string(),
            position: Position::unknown(),
            query: None,
        }
    }
    pub fn not_supported(message: String) -> Self {
        Error {
            error_type: ErrorType::NotSupported,
            message: message,
            position: Position::unknown(),
            query: None,
        }
    }
    pub fn action_not_registered(action: &ActionRequest, namespaces: &Vec<String>) -> Self {
        Error {
            error_type: ErrorType::ActionNotRegistered,
            message: format!(
                "Action '{}' not registered in namespaces {}",
                action.name,
                namespaces.iter().map(|ns| format!("'{}'", ns)).join(", ")
            ),
            position: action.position.clone(),
            query: None,
        }
    }
    pub fn missing_argument(i: usize, name: &str, position: &Position) -> Self {
        Error {
            error_type: ErrorType::ArgumentMissing,
            message: format!("Missing argument #{}:{}", i, name),
            position: position.clone(),
            query: None,
        }
    }
    pub fn conversion_error<W: Display, T: Display>(what: W, to: T) -> Self {
        Error {
            error_type: ErrorType::ConversionError,
            message: format!("Can't convert '{}' to {}", what, to),
            position: Position::unknown(),
            query: None,
        }
    }
    pub fn conversion_error_at_position<W: Display, T: Display>(
        what: W,
        to: T,
        position: &Position,
    ) -> Self {
        Error {
            error_type: ErrorType::ConversionError,
            message: format!("Can't convert '{}' to {}", what, to),
            position: position.clone(),
            query: None,
        }
    }
    pub fn key_parse_error(key: &str, err: &str, position: &Position) -> Self {
        Error {
            error_type: ErrorType::ParseError,
            message: format!("Can't parse key '{}': {}", key, err),
            position: position.clone(),
            query: None,
        }
    }
    pub fn query_parse_error(query: &str, err: &str, position: &Position) -> Self {
        Error {
            error_type: ErrorType::ParseError,
            message: format!("Can't parse query '{}': {}", query, err),
            position: position.clone(),
            query: None,
        }
    }
    pub fn general_error(message: String) -> Self {
        Error {
            error_type: ErrorType::General,
            message: message,
            position: Position::unknown(),
            query: None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.position.is_unknown() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{} at {}", self.message, self.position)
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }
}
