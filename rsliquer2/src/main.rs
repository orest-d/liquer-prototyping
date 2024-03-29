extern crate nom;
extern crate regex;

extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

#[macro_use]
extern crate serde_derive;

mod action_registry;
pub mod cache;
pub mod commands;
mod error;
pub mod metadata;
mod parse;
pub mod plan;
mod query;
pub mod state;
mod value;

use std::collections::HashMap;

use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::{is_alphabetic, is_alphanumeric};
use nom::multi::{many0, separated_list0};
use nom::sequence::pair;
use nom::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Action {
    name: String,
    parameters: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct LogEntry {
    message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Status {
    Ok,
    Failed,
    Pending,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Value {
    None,
    String(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct State {
    query: String,
    log: Vec<LogEntry>,
    status: Status,
    vars: HashMap<String, Value>,
    attributes: HashMap<String, Value>,
    filename: String,
    message: String,
    type_identifier: String,
    cache_allowed: bool,
    is_volatile: bool,
    media_type: String,
}

trait StateType<T> {
    fn identifier() -> String;
    fn default_extension() -> String;
    fn default_media_type() -> String;
    fn is_type_of(data: Value) -> bool;
    fn value_as_bytes(data: Value, format: String) -> Option<(Vec<u8>, String)>;
    fn as_bytes(data: T, format: String) -> Option<(Vec<u8>, String)>;
    fn from_bytes(b: &Vec<u8>, format: String) -> Option<T>;
    fn value_from_bytes(b: &Vec<u8>, format: String) -> Option<Value>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ActionMetadata {
    name: String,
    //    label:String,
    //    module:String,
    doc: String,
    arguments: Vec<ArgumentMetadata>,
    attributes: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ArgumentMetadata {
    name: String,
    data_type: String,
    default: Option<Vec<String>>,
}
/*
trait ArgumentParser<T>{
    fn data_type(&self)->String;
    fn parse<'a>(&self, argv:&'a [String], metadata:&ArgumentMetadata)->Result<(T, &'a [String]), Error>;
}

struct I32ArgumentParser;

impl ArgumentParser<i32> for I32ArgumentParser{
    fn data_type(&self)->String{"i32".to_owned()}
    fn parse<'a>(&self, argv:&'a [String], metadata:&ArgumentMetadata)->Result<(i32, &'a [String]),Error>{
        let (a, rest) =
          argv.split_first().map(|(a,rest)| (a, rest))
          .or_else(||{
            metadata.default.as_ref().and_then(|v| {
                v.first().map(|f| (f, argv))
            })
          }).ok_or(Error::General(format!("Argument {} missing", metadata.name)))?;

        a.parse::<i32>()
        .map(|x| (x,rest))
        .map_err(|e| Error::General(format!("Error parsing argument {};{}",metadata.name,e)))
    }
}
*/

/*
enum ArgumentValue<'a, T>{
    StringValue(&'a str),
    Value(&'a T)
}

trait ArgumentValueDecode<T> {
    fn decode(&self, metadata:&ArgumentMetadata)->Result<T, Error>;
}

impl ArgumentValueDecode<i32> for &str{
    fn decode(&self, metadata:&ArgumentMetadata)->Result<i32, Error>{
        self.parse().map_err(|e| Error::General(format!("Error parsing argument {};{}",metadata.name,e)))
    }
}

impl<'a, T> ArgumentValueDecode<i32> for ArgumentValue<'a, T>{
    fn decode(&self, metadata:&ArgumentMetadata)->Result<i32, Error>{
        match self {
            ArgumentValue::StringValue(s) => s.decode(metadata),
            _ => Err(Error::General(format!("Argument {} type not supported",metadata.name)))
        }
    }
}


trait ArgumentParser<T>{
    fn parse(&mut self, metadata:&ArgumentMetadata)->Result<T, Error>;
}

impl<T,R,AV> ArgumentParser<R> for T where T:Iterator<Item=AV>{
    fn parse(&mut self, metadata:&ArgumentMetadata)->Result<R, Error>{
        Err(Error::General(format!("Argument {} type not supported",metadata.name)))
    }
}

#[derive(Debug, Clone)]
struct Context{
    state:State
}

trait ActionCallable{
    fn call(&self, context:&mut Context, input:&Value, parameters:&Vec<String>)->Option<Value>;
}

struct ActionRegistry{
    registry: HashMap<String,(Box<dyn ActionCallable>, ActionMetadata)>
}
impl ActionRegistry{
    pub fn new()->ActionRegistry{
        ActionRegistry{registry:HashMap::new()}
    }
    pub fn register(&mut self, callable:Box<dyn ActionCallable>, metadata:ActionMetadata){
        self.registry.insert(metadata.name.to_owned(),(callable, metadata));
    }
    pub fn evaluate(&self, context:&mut Context, input:&Value, action:Action)->Option<Value>{
        self.registry.get(&action.name).map(
            |(callable, metadata)| {
                callable.call(context, input, &action.parameters).unwrap()
            }
        )
    }
}

trait Cache{
    fn get(&self, key:&str) -> Option<(Value, State)>;
    fn get_metadata(&self, key:&str) -> Option<State>;
    fn store(&mut self, key:&str, data:Value, metadata:State)->bool;
    fn contains(&self, key:&str)->bool;
}

struct MemoryCache(HashMap<String, (Value, State)>);

impl MemoryCache{
    fn new()->MemoryCache{
        MemoryCache(HashMap::new())
    }
}

impl Cache for MemoryCache{
    fn get(&self, key:&str) -> Option<(Value, State)>{
        self.0.get(key).map(|x|{
            (x.0.clone(),x.1.clone())
        })
    }
    fn get_metadata(&self, key:&str) -> Option<State>{
        self.0.get(key).map(|x| x.1.clone())
    }
    fn store(&mut self, key:&str, data:Value, metadata:State)->bool{
        self.0.insert(key.to_owned(),(data, metadata));
        true
    }
    fn contains(&self, key:&str)->bool{
        self.0.contains_key(key)
    }

}
*/

fn identifier(text: &str) -> IResult<&str, String> {
    let (text, a) = take_while1(|c| is_alphabetic(c as u8) || c == '_')(text)?;
    let (text, b) = take_while(|c| is_alphanumeric(c as u8) || c == '_')(text)?;

    Ok((text, format!("{}{}", a, b)))
}
fn parameter(text: &str) -> IResult<&str, String> {
    let (text, par) = take_while(|c| c != '-' && c != '/')(text)?;

    Ok((text, par.to_owned()))
}

fn parse_action(text: &str) -> IResult<&str, Action> {
    let (text, name) = identifier(text)?;
    let (text, p) = many0(pair(tag("-"), parameter))(text)?;

    Ok((
        text,
        Action {
            name: name,
            parameters: p.iter().map(|x| x.1.to_owned()).collect(),
        },
    ))
}

fn parse_action_path(text: &str) -> IResult<&str, Vec<Action>> {
    separated_list0(tag("/"), parse_action)(text)
}

fn main() {
    println!("Hello, world! {:?}", parse_action_path("aaa-bb-cc/ddd"));

    //  let mut registry = ActionRegistry::new();
}
