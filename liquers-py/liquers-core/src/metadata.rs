use serde_json::{self, Value};

use crate::parse;
use crate::query::Query;

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
pub struct MetadataRecord {
    pub log: Vec<LogEntry>,
    #[serde(with = "query_format")]
    pub query: Query,
    pub status: Status,
    pub type_identifier: String,
    pub message: String,
    pub is_error: bool,
    pub media_type: String,
}

mod query_format {
    use super::*;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(query: &Query, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&query.encode())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Query, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse::parse_query(&s).map_err(de::Error::custom)
    }
}

impl MetadataRecord {
    /// Create a new empty MetadataRecord with default values
    pub fn new() -> MetadataRecord {
        MetadataRecord {
            is_error: false,
            ..Self::default()
        }
    }
    /// Set the query of the MetadataRecord
    pub fn with_query(&mut self, query: Query) -> &mut Self {
        self.query = query;
        self
    }
    /*
    pub fn from_query(query: &str) -> Result<Self, Error> {
        let mut metadata = self::MetadataRecord::new();
        metadata.query = query.to_string();
        Ok(metadata)
    }
    */
}

#[derive(Debug, Clone)]
pub enum Metadata {
    LegacyMetadata(serde_json::Value),
    MetadataRecord(MetadataRecord),
}

impl Metadata {
    pub fn new() -> Metadata {
        Metadata::MetadataRecord(MetadataRecord::new())
    }

    pub fn with_query(&mut self, query: Query) -> &mut Self {
        match self {
            Metadata::LegacyMetadata(serde_json::Value::Object(o)) => {
                o.insert("query".to_string(), Value::String(query.encode()));
                self
            },
            Metadata::MetadataRecord(m) => {
                m.with_query(query);
                self
            },
            Metadata::LegacyMetadata(serde_json::Value::Null) => {
                let mut m = MetadataRecord::new();
                m.query = query;
                *self = Metadata::MetadataRecord(m);
                self
            },

            _ => {
                panic!("Cannot set query on unsupported legacy metadata")
            },
        }
    }

    pub fn from_json(json: &str) -> serde_json::Result<Metadata> {
        match serde_json::from_str::<MetadataRecord>(json) {
            Ok(m) => Ok(Metadata::MetadataRecord(m)),
            Err(_) => match serde_json::from_str::<serde_json::Value>(json) {
                Ok(v) => Ok(Metadata::LegacyMetadata(v)),
                Err(e) => Err(e),
            },
        }
    }

    pub fn from_json_value(json: serde_json::Value) -> serde_json::Result<Metadata> {
        match serde_json::from_value::<MetadataRecord>(json.clone()) {
            Ok(m) => Ok(Metadata::MetadataRecord(m)),
            Err(_) => match serde_json::from_value::<serde_json::Value>(json) {
                Ok(v) => Ok(Metadata::LegacyMetadata(v)),
                Err(e) => Err(e),
            },
        }
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        match self {
            Metadata::LegacyMetadata(v) => serde_json::to_string(v),
            Metadata::MetadataRecord(m) => serde_json::to_string(m),
        }
    }

    pub fn get_media_type(&self) -> String {
        match self {
            Metadata::LegacyMetadata(serde_json::Value::Object(o)) => {
                if let Some(mimetype) = o.get("mimetype") {
                    return mimetype.to_string();
                }
                if let Some(media_type) = o.get("media_type") {
                    return media_type.to_string();
                }
                return "application/octet-stream".to_string();
            }
            Metadata::MetadataRecord(m) => m.media_type.to_string(),
            _ => "application/octet-stream".to_string(),
        }
    }

    pub fn query(&self) -> Result<Query, crate::error::Error> {
        match self {
            Metadata::LegacyMetadata(serde_json::Value::Object(o)) => {
                if let Some(Value::String(query)) = o.get("query") {
                    return parse::parse_query(query);
                }
                return Err(crate::error::Error::General {
                    message: "Query not found".to_string(),
                });
            }
            Metadata::MetadataRecord(m) => Ok(m.query.to_owned()),
            _ => Err(crate::error::Error::General {
                message: "Query not found in unsupported legacy metadata".to_string(),
            }),
        }
    }
}

impl From<MetadataRecord> for Metadata {
    fn from(m: MetadataRecord) -> Self {
        Metadata::MetadataRecord(m)
    }
}
