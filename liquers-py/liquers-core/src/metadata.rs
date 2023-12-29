use serde_json::{self};

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

type Query = String;

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
    pub query: Query,
    pub status: Status,
    pub type_identifier: String,
    pub message: String,
    pub is_error: bool,
    pub mimetype: String,
}

impl MetadataRecord {
    pub fn new() -> MetadataRecord {
        MetadataRecord {
            is_error: false,
            ..Self::default()
        }
    }
    /*
    pub fn from_query(query: &str) -> Result<Self, Error> {
        let mut metadata = self::MetadataRecord::new();
        metadata.query = query.to_string();
        Ok(metadata)
    }
    */

    pub fn cache_key(&self) -> String {
        self.query.to_owned()
    }
}

pub enum Metadata{
    LegacyMetadata(serde_json::Value),
    MetadataRecord(MetadataRecord),
}

impl Metadata{
   pub fn new() -> Metadata{
       Metadata::MetadataRecord(MetadataRecord::new())
   }
   pub fn get_mimetype(&self) -> String{
       match self{
           Metadata::LegacyMetadata(serde_json::Value::Object(o)) => {
                if let Some(mimetype) = o.get("mimetype"){
                     return mimetype.to_string();
                }
                return "application/octet-stream".to_string();
           },
           Metadata::MetadataRecord(m) => m.mimetype.to_string(),
              _ => "application/octet-stream".to_string(),
       }
   } 
}