use crate::{query::Query, error::Error, parse::parse_query};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    None,
    Submitted,
    EvaluatingParent,
    Evaluation,
    EvaluatingDependencies,
    Error,
    Recipe,
    Ready,
    Expired,
    External,
    SideEffect,
}

impl Default for Status {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry {
    message: String,
    message_html: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Metadata {
    pub log: Vec<LogEntry>,
    pub query: Query,
    pub status: Status,
    pub type_identifier: String,
    pub message: String,
    pub is_error: bool,
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata {
            is_error: false,
            ..Self::default()
        }
    }
    pub fn from_query(query: &str) -> Result<Self, Error> {
        let mut metadata = self::Metadata::new();
        metadata.query = parse_query(query)?;
        Ok(metadata)
    }

    pub fn cache_key(&self)->String{
        self.query.encode()
    }
}
