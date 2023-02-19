use crate::error::Error;
use crate::state::State;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;
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
    pub fn position(&self) -> Position {
        match self {
            CommandParameter::String(_, p) => p.clone(),
            CommandParameter::Link(_, p) => p.clone(),
            CommandParameter::Value(_) => Position::unknown(),
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
    fn try_from_state_check(
        value: &State<V>,
        context: &mut impl Context<V>,
        check: bool,
    ) -> Result<Self, Error>{
        Self::try_from_check(&CommandParameter::Value(value.data.clone()), context, check)
    }
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

pub trait Command<V,C>
where
    V: ValueInterface, C: Context<V>
{
    fn call_command(
        &mut self,
        state:State<V>,
        parameters: &[CommandParameter<V>],
        check:bool,
        context: &mut C,
    ) -> Result<V, Error>;
    fn execute_action(
        &mut self,
        state:State<V>,
        action: &ActionRequest,
        context: &mut C,
    ) -> Result<V, Error> {
        let mut par: Vec<_> = action
            .parameters
            .iter()
            .map(|x| CommandParameter::from_action_parameter(x))
            .collect();
        self.call_command(state, par.as_slice(), false, context)
    }
}

impl<F, V, R, C> Command<V,C> for F
where
    F: FnMut() -> R,
    V: ValueInterface + From<R>,
    C: Context<V>
{
    fn call_command(
        &mut self,
        state:State<V>,
        parameters: &[CommandParameter<V>],
        check:bool,
        context: &mut C,
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


pub struct Command1<S,R,F> where F:FnMut(S) -> R{
    f:F,
    state:PhantomData<S>,
    result:PhantomData<R>
}

impl<S,R,F> From<F> for Command1<S,R,F> where F:FnMut(S) -> R{
    fn from(f: F) -> Self {
        Command1 { f, state: Default::default(), result: Default::default() }
    }
}

pub struct Command2<S,A1,R,F> where F:FnMut(S,A1) -> R{
    f:F,
    state:PhantomData<S>,
    arg1:PhantomData<A1>,
    result:PhantomData<R>
}

impl<S,A1,R,F> From<F> for Command2<S,A1,R,F> where F:FnMut(S,A1) -> R{
    fn from(f: F) -> Self {
        Command2 { f, state: Default::default(), arg1:Default::default(), result: Default::default() }
    }
}

impl<V, S, R, F, C> Command<V, C> for Command1<S,R,F>
where
    F: FnMut(S) -> R,
    V: ValueInterface + From<R>,
    S: TryFromCommandParameter<V>,
    C: Context<V>
{
    fn call_command(
        &mut self,
        state:State<V>,
        parameters: &[CommandParameter<V>],
        check:bool,
        context: &mut C,
    ) -> Result<V, crate::error::Error> {
        if parameters.is_empty() {
            if check{
                S::try_from_state_check(&state, context, true)?;
                Ok(V::none())
            }
            else{
                Ok(V::from((self.f)(S::try_from_state_check(&state, context, false)?)))
            }
        } else {
            Err(Error::ParameterError {
                message: format!("Too many parameters"),
                position: Position::unknown(),
            })
        }
    }
}

impl<V, S, A1, R, F, C> Command<V, C> for Command2<S,A1,R,F>
where
    F: FnMut(S,A1) -> R,
    V: ValueInterface + From<R>,
    S: TryFromCommandParameter<V>,
    A1: TryFromCommandParameter<V>,
    C: Context<V>
{
    fn call_command(
        &mut self,
        state:State<V>,
        parameters: &[CommandParameter<V>],
        check:bool,
        context: &mut C,
    ) -> Result<V, crate::error::Error> {
        if parameters.len() == 1 {
            if check{
                S::try_from_state_check(&state, context, true)?;
                A1::try_from_check(&parameters[0], context, true)?;
                Ok(V::none())
            }
            else{
                Ok(V::from((self.f)(
                    S::try_from_state_check(&state, context, false)?,
                    A1::try_from_check(&parameters[0], context, false)?,
            )))
            }
        } else {
            Err(Error::ParameterError {
                message: format!("Exactly one parameter expected, {} provided", parameters.len()),
                position: if parameters.is_empty() {Position::unknown()} else {parameters[1].position()} ,
            })
        }
    }
}



struct CommandExcecutorRegistry<V, C> where C:Context<V>, V:ValueInterface{
    reg:HashMap<String, HashMap<String, Box<dyn Command<V,C>>>>
}

impl<V,C> CommandExcecutorRegistry<V,C> where C:Context<V>, V:ValueInterface{
    fn new()->Self{
        CommandExcecutorRegistry{
            reg:HashMap::new()
        }
    }
    fn exists(self, name: &str, ns:&str) -> bool{
        self.reg.get(name).map(|x| x.contains_key(ns)).unwrap_or(false)
    }

    /*        
    fn register(&mut self, name:&str, ns:&str, command: Box<dyn Command<V, C>>) -> Result<(),Error>{
        self.reg.get_mut(k)
        if self.reg.contains_key(name){
            if 
            Err(Error::CommandAlreadyRegistered { message: format!() })
        }
    }
    */
}

impl<V, C> Default for CommandExcecutorRegistry<V,C>
where C:Context<V>, V:ValueInterface
{
    fn default() -> Self {
        Self::new()
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
            State::<Value>::new(),
            &a,
            &mut context,
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
            &mut context,
        );
        assert_eq!(result.unwrap().try_into_i32().unwrap(), 123);
        assert!(called);
        Ok(())
    }
    #[test]
    fn test_command1_i32() -> Result<(), Box<dyn std::error::Error>> {
        let q = crate::parse::parse_query("abc")?;
        let a = q.action().unwrap();
        let mut called = false;
        assert!(!called);
        let mut f = |x:i32| {
            called = true;
            123+x
        };
        let mut context = DummyContext;
        let value = Value::from(234);
        let mut cmd = Command1::from(&mut f);
        let state = State::new().with_data(value);
        let result = cmd.execute_action(
            state,
            &a,
            &mut context,
        );
        assert_eq!(result.unwrap().try_into_i32().unwrap(), 357);
        assert!(called);
        Ok(())
    }
    #[test]
    fn test_command2_i32() -> Result<(), Box<dyn std::error::Error>> {
        let q = crate::parse::parse_query("abc-321")?;
        let a = q.action().unwrap();
        let mut called = false;
        assert!(!called);
        let mut f = |x:i32, y:i32| {
            called = true;
            123+x+y
        };
        let mut context = DummyContext;
        let value = Value::from(234);
        let mut cmd = Command2::from(&mut f);
        let state = State::new().with_data(value);
        let result = cmd.execute_action(
            state,
            &a,
            &mut context,
        );
        assert_eq!(result.unwrap().try_into_i32().unwrap(), 678);
        assert!(called);
        Ok(())
    }

}