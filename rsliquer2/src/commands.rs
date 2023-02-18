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

pub trait TryFromCommandParameter<V>
where
    V: ValueInterface,
    Self: Sized,
{
    fn try_from_check(
        value: &CommandParameter<V>,
        context: &mut impl Context<V>,
        check: bool,
    ) -> Result<Self, Error>;
    fn try_from(value: &CommandParameter<V>, context: &mut impl Context<V>) -> Result<Self, Error> {
        Self::try_from_check(value, context, false)
    }
}

impl<V: ValueInterface> TryFromCommandParameter<V> for Arc<V> {
    fn try_from_check(
        value: &CommandParameter<V>,
        context: &mut impl Context<V>,
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

impl<V: ValueInterface> TryFromCommandParameter<V> for State<V> {
    fn try_from_check(
        value: &CommandParameter<V>,
        context: &mut impl Context<V>,
        check: bool,
    ) -> Result<Self, Error> {
        match value {
            CommandParameter::String(s, _) => Ok(State::new().with_string(s)),
            CommandParameter::Link(q, p) => {
                if check {
                    Ok(State::new())
                } else {
                    context.evaluate_parameter_link(q, p)
                }
            }
            CommandParameter::Value(v) => Ok(State::new().with_data((**v).clone())),
        }
    }
}

impl<V: ValueInterface> TryFromCommandParameter<V> for Option<String> {
    fn try_from_check(
        value: &CommandParameter<V>,
        context: &mut impl Context<V>,
        check: bool,
    ) -> Result<Self, Error> {
        match value {
            CommandParameter::String(s, _) => Ok(if s.is_empty() {
                None
            } else {
                Some(s.to_owned())
            }),
            CommandParameter::Link(q, p) => {
                if check {
                    Ok(None)
                } else {
                    context
                        .evaluate_parameter_link(q, p)
                        .and_then(|x| x.data.try_into_string_option())
                }
            }
            CommandParameter::Value(v) => v.try_into_string_option(),
        }
    }
}

impl<V: ValueInterface> TryFromCommandParameter<V> for i32 {
    fn try_from_check(
        value: &CommandParameter<V>,
        context: &mut impl Context<V>,
        check: bool,
    ) -> Result<Self, Error> {
        match value {
            CommandParameter::String(s, p) => s.parse().map_err(|e| Error::ParameterError {
                message: format!("Integer parse error:{e}"),
                position: p.clone(),
            }),
            CommandParameter::Link(q, p) => {
                if check {
                    Ok(0)
                } else {
                    context.evaluate_parameter_link(q, p).and_then(|x| x.data.try_into_i32())
                }
            }
            CommandParameter::Value(v) => v.try_into_i32(),
        }
    }
}

/*
impl<V: ValueInterface + TryInto<i32>, C: Context<V>> TryFromCommandParameter<V, C> for i32 {
    fn try_from_check(
        value: &CommandParameter<V>,
        context: &mut C,
        check: bool,
    ) -> Result<Self, Error> {
        match value {
            CommandParameter::String(s, position) => i32::from_str(&s).map_err(|e| Error::ParameterError { message:format!("Integer parameter expected; {e}"), position:position.clone() }),
            CommandParameter::Link(q, p) => {
                if check {
                    Ok(0)
                } else {
                    context
                        .evaluate_parameter_link(q, p).and_then(|x|{
                            (*x.data).try_into().map_err(|e| Error::ParameterError { message:format!("Integer parameter expected"), position:Position::unknown() })
                        })
                }
            }
            CommandParameter::Value(v) => (**v).try_into().map_err(|e| Error::ParameterError { message:format!("Integer parameter expected"), position:Position::unknown() }),
        }
    }
}
*/

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
    fn evaluate(&mut self, query: &Query) -> Result<State<V>, Error>;
    fn evaluate_parameter_link(
        &mut self,
        query: &Query,
        position: &Position,
    ) -> Result<State<V>, Error> {
        self.evaluate(query)
    }
}
struct DummyContext;

impl<V: ValueInterface> Context<V> for DummyContext {
    fn evaluate(&mut self, query: &Query) -> Result<State<V>, Error> {
        Err(Error::NotSupported {
            message: format!("Dummy context does not support evaluation."),
        })
    }
}

pub trait Command<V>
where
    V: ValueInterface,
{
    fn call_command(
        &mut self,
        state:State<V>,
        parameters: &[CommandParameter<V>],
        check:bool,
        context: impl Context<V>,
    ) -> Result<V, Error>;
    fn execute_action(
        &mut self,
        state:State<V>,
        action: &ActionRequest,
        context: impl Context<V>,
    ) -> Result<V, Error> {
        let mut par: Vec<_> = action
            .parameters
            .iter()
            .map(|x| CommandParameter::from_action_parameter(x))
            .collect();
        self.call_command(state, par.as_slice(), false, context)
    }
}

impl<F, V, R> Command<V> for F
where
    F: FnMut() -> R,
    V: ValueInterface + From<R>
{
    fn call_command(
        &mut self,
        state:State<V>,
        parameters: &[CommandParameter<V>],
        check:bool,
        context: impl Context<V>,
    ) -> Result<V, Error> {
        if parameters.is_empty() {
            if check{
                Ok(V::none())
            }
            else{
                Ok(V::from(self()))
            }
        } else {
            Err(Error::ParameterError {
                message: format!("Too many parameters"),
                position: Position::unknown(),
            })
        }
    }
}

/*
pub struct<S,R,F:FnMut(S) -> R> Command1(F);

impl<V, S, R, F:FnMut(S) -> R> Command<V> for Command1<S,R,F>
where
    V: ValueInterface + From<R>,
    S: TryFrom<V>
{
    fn call_command(
        &mut self,
        state:State<V>,
        parameters: &[CommandParameter<V>],
        check:bool,
        context: impl Context<V>,
    ) -> Result<V, Error> {
        if parameters.is_empty() {
            if check{
                S::try_from(state.data)?;
                Ok(V::none())
            }
            else{
                Ok(V::from(self(S::try_from(state.data))))
            }
        } else {
            Err(Error::ParameterError {
                message: format!("Too many parameters"),
                position: Position::unknown(),
            })
        }
    }
}
*/

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
            State::<Value>::new(),
            &a,
            context,
        );
        assert!(called);
        Ok(())
    }

    #[test]
    fn test_command_i32() -> Result<(), Box<dyn std::error::Error>> {
        let q = crate::parse::parse_query("abc")?;
        let a = q.action().unwrap();
        let mut called = false;
        assert!(!called);
        let mut f = || {
            called = true;
            123
        };
        let mut context = DummyContext;
        let result = (&mut f).execute_action(
            State::<Value>::new(),
            &a,
            context,
        );
        assert_eq!(result.unwrap().try_into_i32().unwrap(), 123);
        assert!(called);
        Ok(())
    }
}