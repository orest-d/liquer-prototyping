use std::result::Result;
use std::fmt::Display;
use crate::error::Error;
use itertools::Itertools;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position{
    pub offset:usize,
    pub line:u32,
    pub column:usize
}

impl Position{
    pub fn unknown()->Position{
        Position{offset:0, line:0, column:0}
    }
}
impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line==0{
            write!(f, "(unknown position)")
        }
        else if self.line>1{
            write!(f, "line {}, position {}", self.line, self.column)
        }
        else{
            write!(f, "position {}", self.column)
        }
    }
}

pub fn encode_token<S: AsRef<str>>(text:S)->String{
    format!("{}",text.as_ref())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionParameter{
    String(String, Position),
    Link(String, Position), // TODO: Link should have a query inside 
}

impl ActionParameter{
    pub fn new(parameter:&str)->ActionParameter{
        ActionParameter::String(parameter.to_owned(), Position::unknown())
    }
    pub fn new_parsed(parameter:String, position:Position)->ActionParameter{
        ActionParameter::String(parameter, position)
    }
    pub fn encode(&self)->String{
        match self {
            Self::String(s, _) => encode_token(s),
            Self::Link(query, _) => format!("~X~{}~E", query) // TODO: query.encode()
        }
    }
}

impl Display for ActionParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceName{
    pub name:String,
    pub position:Position,
}

impl ResourceName {
    pub fn encode(&self)->&str{
        &self.name
    }
}

impl Display for ResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionRequest{
    pub name:String,
    pub position:Position,
    pub parameters: Vec<ActionParameter>
}

impl ActionRequest{
    pub fn encode(&self)->String{
        if self.parameters.len() == 0{
            return self.name.to_owned()
        }
        else{
            format!("{}-{}", self.name, self.parameters.iter().map(|x| x.encode()).join("-"))
        }
    }
}

impl Display for ActionRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}


/// Header of a query segment - both resource and transformation query.
/// Header may contain name (string), level (integer) and parameters (list of strings).
/// The header parameters may influence how the query is interpreted.
/// The interpretation of the header parameters depends on the context object.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct SegmentHeader{
    name: String,
    level: usize,
    parameters: Vec<String>,
    resource: bool,
    position: Position,
}

impl SegmentHeader{
    /// Returns true if the header does not contain any data,
    /// I.e. trivial header has no name, level is 1 and no parameters.
    /// Trivial header can be both for resource and query, it does not depend on the resource flags.
    pub fn is_trivial(&self) -> bool {
        self.name.is_empty() && self.level == 1 && self.parameters.len() == 0
    }

    pub fn encode(&self) -> String {
        let mut encoded:String = std::iter::repeat("-").take(self.level+1).collect();
        if self.resource{
            encoded.push('R');
        }
        encoded.push_str(&self.name);
        if !self.parameters.is_empty(){
            //assert len(self.name) > 0 or self.resource
            for parameter in self.parameters.iter(){
                encoded.push('-');
                encoded.push_str(&parameter);
            }
        }
        encoded
    }
}

impl Display for SegmentHeader{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.encode())
    }
}
    


#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn encode_link_action_parameter() -> Result<(), Box<dyn std::error::Error>>{
        let ap = ActionParameter::Link("hello".to_string(), Position::unknown());
        assert_eq!(ap.encode(),"~X~hello~E");
        Ok(())
    }

    #[test]
    fn encode_action_request() -> Result<(), Box<dyn std::error::Error>>
    {
        let a = ActionRequest{name:"action".to_owned(), position: Position::unknown(), parameters:vec![]};
        assert_eq!(a.encode(), "action");
        let a = ActionRequest{name:"action".to_owned(), position: Position::unknown(),
          parameters:vec![
            ActionParameter::Link("hello".to_string(), Position::unknown()),
            ActionParameter::String("world".to_string(), Position::unknown()),
          ]};
        assert_eq!(a.encode(), "action-~X~hello~E-world");
        Ok(())
    }
}