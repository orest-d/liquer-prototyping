use itertools::Itertools;
use std::fmt::Display;
use std::ops::Add;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub offset: usize,
    pub line: u32,
    pub column: usize,
}

#[allow(dead_code)]
impl Position {
    pub fn new(offset: usize, line: u32, column: usize) -> Self {
        Position {
            offset: offset,
            line: line,
            column: column,
        }
    }
    pub fn unknown() -> Position {
        Position {
            offset: 0,
            line: 0,
            column: 0,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::unknown()
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 {
            write!(f, "(unknown position)")
        } else if self.line > 1 {
            write!(f, "line {}, position {}", self.line, self.column)
        } else {
            write!(f, "position {}", self.column)
        }
    }
}

pub fn encode_token<S: AsRef<str>>(text: S) -> String {
    format!("{}", text.as_ref())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionParameter {
    String(String, Position),
    Link(String, Position), // TODO: Link should have a query inside
}

#[allow(dead_code)]
impl ActionParameter {
    pub fn new_string(parameter: String) -> ActionParameter {
        ActionParameter::String(parameter, Position::unknown())
    }
    pub fn new_link(parameter: String) -> ActionParameter {
        ActionParameter::Link(parameter, Position::unknown())
    }
    pub fn with_position(self, position: Position) -> Self {
        match self {
            Self::String(s, _) => Self::String(s, position),
            Self::Link(query, _) => Self::Link(query, position), // TODO: query.encode()
        }
    }
    pub fn encode(&self) -> String {
        match self {
            Self::String(s, _) => encode_token(s),
            Self::Link(query, _) => format!("~X~{}~E", query), // TODO: query.encode()
        }
    }
}

impl Display for ActionParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceName {
    pub name: String,
    pub position: Position,
}

#[allow(dead_code)]
impl ResourceName {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            position: Position::unknown(),
        }
    }
    pub fn with_position(self, position: Position) -> Self {
        Self {
            position: position,
            ..self
        }
    }
    pub fn encode(&self) -> &str {
        &self.name
    }
}

impl Display for ResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ActionRequest {
    pub name: String,
    pub parameters: Vec<ActionParameter>,
    pub position: Position,
}

#[allow(dead_code)]
impl ActionRequest {
    pub fn new(name: String) -> ActionRequest {
        ActionRequest {
            name: name,
            ..Default::default()
        }
    }
    pub fn with_position(self, position: Position) -> Self {
        Self {
            position: position,
            ..self
        }
    }
    pub fn with_parameters(self, parameters: Vec<ActionParameter>) -> Self {
        Self {
            parameters: parameters,
            ..self
        }
    }
    pub fn encode(&self) -> String {
        if self.parameters.len() == 0 {
            return self.name.to_owned();
        } else {
            format!(
                "{}-{}",
                self.name,
                self.parameters.iter().map(|x| x.encode()).join("-")
            )
        }
    }
}

impl Display for ActionRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HeaderParameter {
    pub value: String,
    pub position: Position,
}

#[allow(dead_code)]
impl HeaderParameter {
    pub fn new(value: String) -> HeaderParameter {
        HeaderParameter {
            value: value,
            ..Default::default()
        }
    }
    pub fn with_position(self, position: Position) -> Self {
        Self {
            value: self.value,
            position: position,
        }
    }
    pub fn encode(&self) -> &str {
        &self.value
    }
}

impl Display for HeaderParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Header of a query segment - both resource and transformation query.
/// Header may contain name (string), level (integer) and parameters (list of strings).
/// The header parameters may influence how the query is interpreted.
/// The interpretation of the header parameters depends on the context object.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct SegmentHeader {
    name: String,
    level: usize,
    parameters: Vec<HeaderParameter>,
    resource: bool,
    position: Position,
}

#[allow(dead_code)]
impl SegmentHeader {
    /// Returns true if the header does not contain any data,
    /// I.e. trivial header has no name, level is 0 and no parameters.
    /// Trivial header can be both for resource and query, it does not depend on the resource flags.
    pub fn is_trivial(&self) -> bool {
        self.name.is_empty() && self.level == 0 && self.parameters.len() == 0
    }

    pub fn new() -> SegmentHeader {
        SegmentHeader {
            name: "".to_owned(),
            level: 0,
            parameters: vec![],
            resource: false,
            position: Position::unknown(),
        }
    }
    pub fn with_position(self, position: Position) -> Self {
        Self {
            position: position,
            ..self
        }
    }

    pub fn encode(&self) -> String {
        let mut encoded: String = std::iter::repeat("-").take(self.level + 1).collect();
        if self.resource {
            encoded.push('R');
        }
        encoded.push_str(&self.name);
        if !self.parameters.is_empty() {
            //assert len(self.name) > 0 or self.resource
            for parameter in self.parameters.iter() {
                encoded.push('-');
                encoded.push_str(parameter.encode());
            }
        }
        encoded
    }
}

impl Display for SegmentHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

/// Query segment representing a transformation, i.e. a sequence of actions applied to a state.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct TransformQuerySegment {
    header: Option<SegmentHeader>,
    query: Vec<ActionRequest>,
    filename: Option<ResourceName>,
}

#[allow(dead_code)]
impl TransformQuerySegment {
    pub fn predecessor(&self) -> (Option<TransformQuerySegment>, Option<TransformQuerySegment>) {
        if let Some(filename) = &self.filename {
            (
                Some(TransformQuerySegment {
                    header: self.header.clone(),
                    query: self.query.clone(),
                    filename: None,
                }),
                Some(TransformQuerySegment {
                    header: self.header.clone(),
                    query: vec![],
                    filename: Some(filename.clone()),
                }),
            )
        } else {
            if self.query.is_empty() {
                (None, None)
            } else {
                let mut q = vec![];
                self.query[0..self.query.len() - 1].clone_into(&mut q);
                (
                    Some(TransformQuerySegment {
                        header: self.header.clone(),
                        query: q,
                        filename: None,
                    }),
                    Some(TransformQuerySegment {
                        header: self.header.clone(),
                        query: vec![self.query.last().unwrap().clone()],
                        filename: None,
                    }),
                )
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_empty() && self.filename.is_none()
    }

    pub fn is_filename(&self) -> bool {
        self.query.is_empty() && self.filename.is_some()
    }

    pub fn is_action_request(&self) -> bool {
        self.query.len() == 1 && self.filename.is_none()
    }

    pub fn action(&self) -> Option<ActionRequest> {
        if self.is_action_request() {
            Some(self.query[0].clone())
        } else {
            None
        }
    }

    pub fn encode(&self) -> String {
        let pure_query = self.query.iter().map(|x| x.encode()).join("/");
        let query = if let Some(filename) = &self.filename {
            if pure_query.is_empty() {
                filename.encode().to_owned()
            } else {
                format!("{}/{}", pure_query, filename.encode())
            }
        } else {
            pure_query
        };

        if let Some(header) = &self.header {
            if query.is_empty() {
                header.encode()
            } else {
                format!("{}/{}", header.encode(), query)
            }
        } else {
            query
        }
    }
}

impl Add for TransformQuerySegment {
    type Output = TransformQuerySegment;

    fn add(self, rhs: Self) -> Self::Output {
        let mut q = self.query.clone();
        q.extend(rhs.query.iter().map(|x| x.clone()));
        TransformQuerySegment {
            header: self.header.clone(),
            query: q,
            filename: rhs.filename.clone(),
        }
    }
}

impl Add<Option<TransformQuerySegment>> for TransformQuerySegment {
    type Output = TransformQuerySegment;

    fn add(self, rhs: Option<TransformQuerySegment>) -> Self::Output {
        match rhs {
            Some(x) => self + x,
            None => self,
        }
    }
}

impl Display for TransformQuerySegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

/// Query segment representing a resource, i.e. path to a file in a store.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ResourceQuerySegment {
    header: Option<SegmentHeader>,
    query: Vec<ResourceName>,
}

#[allow(dead_code)]
impl ResourceQuerySegment {
    /// Return resource query position
    pub fn position(&self) -> Position {
        if let Some(header) = &self.header {
            header.position.to_owned()
        } else {
            if self.query.is_empty() {
                Position::unknown()
            } else {
                self.query[0].position.to_owned()
            }
        }
    }

    /// Path to the resource as a string.
    /// This is typically interpreted as a resource key in a Store object.
    pub fn path(&self) -> String {
        self.query.iter().map(|x| x.encode()).join("/")
    }

    pub fn encode(&self) -> String {
        let mut rqs = self.header.as_ref().map_or("".to_owned(), |x| x.encode());
        if !rqs.is_empty() {
            rqs.push('/');
        }
        if self.query.is_empty() {
            rqs
        } else {
            let query = self.path();
            format!("{rqs}{query}")
        }
    }
}

impl Display for ResourceQuerySegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_link_action_parameter() -> Result<(), Box<dyn std::error::Error>> {
        let ap = ActionParameter::Link("hello".to_string(), Position::unknown());
        assert_eq!(ap.encode(), "~X~hello~E");
        Ok(())
    }

    #[test]
    fn encode_action_request() -> Result<(), Box<dyn std::error::Error>> {
        let a = ActionRequest {
            name: "action".to_owned(),
            position: Position::unknown(),
            parameters: vec![],
        };
        assert_eq!(a.encode(), "action");
        let a = ActionRequest::new("action1".to_owned());
        assert_eq!(a.encode(), "action1");
        let a = ActionRequest {
            name: "action".to_owned(),
            position: Position::unknown(),
            parameters: vec![
                ActionParameter::Link("hello".to_string(), Position::unknown()),
                ActionParameter::String("world".to_string(), Position::unknown()),
            ],
        };
        assert_eq!(a.encode(), "action-~X~hello~E-world");
        let a = ActionRequest::new("action1".to_owned()).with_parameters(vec![
            ActionParameter::new_link("hello".to_owned()),
            ActionParameter::new_string("world".to_owned()),
        ]);
        assert_eq!(a.encode(), "action1-~X~hello~E-world");
        Ok(())
    }

    #[test]
    fn encode_segment_header() -> Result<(), Box<dyn std::error::Error>> {
        let head = SegmentHeader::new();
        assert_eq!(head.encode(), "-");
        Ok(())
    }
}
