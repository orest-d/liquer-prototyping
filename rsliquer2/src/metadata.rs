use crate::query::Query;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status{
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
    SideEffect
}

impl Default for Status {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEntry{
    message: String,
    message_html: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Metadata{
    log: Vec<LogEntry>,
    query: Option<Query>,
    status: Status,
    type_identifier: String,
    message: String,
    is_error: bool
}

impl Metadata {
    pub fn new() -> Metadata{
        Metadata { is_error: false, ..Self::default() }
    }

}
