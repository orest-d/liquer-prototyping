use nom_locate::position;

use crate::error::Error;
use crate::state::State;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use std::sync::Arc;

use crate::metadata::Metadata;
use crate::query::{ActionParameter, ActionRequest, Position, Query};
use crate::value::*;
/*
pub struct CommandParameter<V>
where
    V: ValueInterface,
{
    value: Arc<V>,
    position: Position,
}


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
    Value(Arc<V>),
}

impl<V> CommandParameter<V>
where
    V: ValueInterface,
{
    pub fn from_action_parameter(action: &ActionParameter) -> Self {
        match action {
            ActionParameter::String(s, p) => CommandParameter::String(s.to_owned(), p.to_owned()),
            ActionParameter::Link(q, p) => CommandParameter::Link(q.to_owned(), p.to_owned()),
        }
    }
}

pub trait TryFromCommandParameter<V, C>
where
    V: ValueInterface,
    C: Context<V>,
    Self: Sized,
{
    fn try_from_check(
        value: &CommandParameter<V>,
        context: &mut C,
        check: bool,
    ) -> Result<Self, Error>;
    fn try_from(value: &CommandParameter<V>, context: &mut C) -> Result<Self, Error> {
        Self::try_from_check(value, context, false)
    }
}

impl<V: ValueInterface, C: Context<V>> TryFromCommandParameter<V, C> for Arc<V> {
    fn try_from_check(
        value: &CommandParameter<V>,
        context: &mut C,
        check: bool,
    ) -> Result<Self, Error> {
        match value {
            CommandParameter::String(s, _) => Ok(Arc::new(V::new(s))),
            CommandParameter::Link(q, p) => {
                if check {
                    Ok(Arc::new(V::none()))
                } else {
                    context
                        .evaluate_parameter_link(q, p)
                        .map(|x| x.data.clone())
                }
            }
            CommandParameter::Value(v) => Ok(v.clone()),
        }
    }
}

/*
impl<'s, V: ValueInterface, C:Context> TryFromCommandParameter<'s, V, C, _> for &'s mut C{
    fn try_from_check(value: &CommandParameter<Value>, context:&'s mut C, check:bool) -> Result<'s+Self, Error>{
        Ok(context)
    }
}
*/
/*
impl TryFrom<CommandParameter<Value>> for i32 {
    type Error = Error;
    fn try_from(value: CommandParameter<Value>) -> Result<Self, Self::Error> {
        match value {
            CommandParameter::String(s, position) => {
                i32::from_str(&s).map_err(|e| Error::ParameterError { message:format!("Integer parameter expected; {e}"), position:position.clone() })
            },
            CommandParameter::Link(q, position) => Err(Error::ParameterError { message:format!("Link not supported: '{}'",q.encode()), position:position.clone() }),
            CommandParameter::Value(v) => i32::try_from(&*v),
        }
    }
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
*/
pub trait Context<V: ValueInterface> {
    fn evaluate_parameter_link(
        &mut self,
        query: &Query,
        position: &Position,
    ) -> Result<State<V>, Error>;
}
struct DummyContext;

impl<V: ValueInterface> Context<V> for DummyContext {
    fn evaluate_parameter_link(
        &mut self,
        query: &Query,
        position: &Position,
    ) -> Result<State<V>, Error> {
        todo!()
    }
}

pub trait Command<V>
where
    V: ValueInterface,
{
    fn call_command(
        &mut self,
        state_data: Arc<V>,
        state_metadata: Arc<Metadata>,
        parameters: &[CommandParameter<V>],
        context: impl Context<V>,
    ) -> Result<V, Error>;
    fn execute_action(
        &mut self,
        state_data: Arc<V>,
        state_metadata: Arc<Metadata>,
        action: &ActionRequest,
        context: impl Context<V>,
    ) -> Result<V, Error> {
        let mut par: Vec<_> = action
            .parameters
            .iter()
            .map(|x| CommandParameter::from_action_parameter(x))
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
        context: impl Context<V>,
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
