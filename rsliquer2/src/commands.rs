use crate::error::Error;
use std::sync::Arc;
use std::time::Instant;

use crate::metadata::Metadata;
use crate::query::{ActionParameter, ActionRequest, Position, Query};
use crate::value::ValueInterface;
/*
pub struct CommandParameter<V>
where
    V: ValueInterface,
{
    value: Arc<V>,
    position: Position,
}

pub trait Context {}
struct DummyContext;

impl Context for DummyContext {}


pub enum Version{
    Unspecified,
    Any,
    Latest,
    Exists,
    NotExists,
    Timestamp(String),
    Md5(String),
}

struct ArgumentInfo{
    name: String,
    optional: bool,
    type_name: String,
    multiple: bool,
}

struct CommandMetadata{
    name:String,
    module:String,
    doc:String,
    state_argument:ArgumentInfo,
    arguments:Vec<ArgumentInfo>,
    version:Version
}
*/

pub enum CommandParameter<V>
where
    V: ValueInterface,
{
    String(String, Position),
    Link(Query, Position),
    Value(Arc<V>, Position),
}

impl<V> CommandParameter<V>
where
    V: ValueInterface,
{
    //    pub fn convert_into<T,C:Context>(&self, )
}

impl<V: ValueInterface> From<String> for CommandParameter<V> {
    fn from(value: String) -> Self {
        CommandParameter::String(value, Position::unknown())
    }
}

impl<V: ValueInterface> From<&ActionParameter> for CommandParameter<V> {
    fn from(value: &ActionParameter) -> Self {
        match value {
            ActionParameter::String(value, position) => {
                CommandParameter::String(value.to_owned(), position.to_owned())
            }
            ActionParameter::Link(query, position) => {
                CommandParameter::Link(query.to_owned(), position.to_owned())
            }
        }
    }
}

pub trait Context {}
struct DummyContext;

impl Context for DummyContext {}

pub trait Command<V>
where
    V: ValueInterface,
{
    fn call_command(
        &mut self,
        state_data: Arc<V>,
        state_metadata: Arc<Metadata>,
        parameters: &[CommandParameter<V>],
        context: impl Context,
    ) -> Result<V, Error>;
    fn execute_action(
        &mut self,
        state_data: Arc<V>,
        state_metadata: Arc<Metadata>,
        action: &ActionRequest,
        context: impl Context,
    ) -> Result<V, Error> {
        let mut par: Vec<_> = action
            .parameters
            .iter()
            .map(|x| CommandParameter::from(x))
            .collect();
        self.call_command(state_data, state_metadata, par.as_slice(), context)
    }
}

impl<F, V> Command<V> for F
where
    F: FnMut() -> (),
    V: ValueInterface,
{
    fn call_command(
        &mut self,
        state_data: Arc<V>,
        state_metadata: Arc<Metadata>,
        parameters: &[CommandParameter<V>],
        context: impl Context,
    ) -> Result<V, Error> {
        if parameters.is_empty() {
            self();
            Ok(V::none())
        } else {
            Err(Error::ParameterError {
                message: format!("Too many parameters"),
                position: Position::unknown(),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::value::Value;

    use super::*;

    #[test]
    fn test_command() -> Result<(), Box<dyn std::error::Error>> {
        let q = crate::parse::parse_query("abc")?;
        let a = q.action().unwrap();
        let mut called = false;
        assert!(!called);
        let mut f = || {
            called = true;
        };
        let mut context = DummyContext;
        (&mut f).execute_action(
            Arc::new(Value::none()),
            Arc::new(Metadata::new()),
            &a,
            context,
        );
        assert!(called);
        Ok(())
    }
}
