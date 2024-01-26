use std::collections::HashMap;
use std::marker::PhantomData;
use std::result;

use crate::command_metadata::{CommandKey, CommandMetadata};
use crate::error::{Error, ErrorType};
use crate::plan::{Parameter, ResolvedParameters};
use crate::query::Position;
use crate::state::State;
use crate::value::ValueInterface;

pub struct NoInjection;

pub struct CommandArguments<'i, Injection> {
    pub parameters: ResolvedParameters,
    pub injection: &'i Injection,
    pub action_position: Position,
    pub argument_number: usize,
}

impl<'i, Injection> CommandArguments<'i, Injection> {
    pub fn new(parameters: ResolvedParameters, injection: &'i Injection) -> Self {
        CommandArguments {
            parameters,
            injection,
            action_position: Position::unknown(),
            argument_number: 0,
        }
    }
    pub fn has_no_parameters(&self) -> bool {
        self.parameters.parameters.is_empty()
    }
    pub fn get<T: FromParameter<T, Injection>>(&mut self) -> Result<T, Error> {
        if let Some(p) = self.parameters.parameters.get(self.argument_number) {
            self.argument_number += 1;
            T::from_parameter(p, self.injection)
        } else {
            Err(Error::missing_argument(
                self.argument_number,
                "?",
                &self.action_position,
            ))
        }
    }
}

/// Command trait
/// This trait encapsulates a command that can be executed,
/// typically a function
pub trait Command<Injection, V: ValueInterface> {
    fn execute(
        &mut self,
        state: &State<V>,
        arguments: &mut CommandArguments<Injection>,
    ) -> Result<V, Error>;

    /// Returns the default metadata of the command
    /// This may be modified or overriden inside the command registry
    fn command_metadata(&self) -> Option<CommandMetadata> {
        None
    }
}

impl<F, R, Injection, V> Command<Injection, V> for F
where
    F: FnMut() -> R,
    V: ValueInterface + From<R>,
{
    fn execute(
        &mut self,
        _state: &State<V>,
        arguments: &mut CommandArguments<'_, Injection>,
    ) -> Result<V, Error> {
        if arguments.has_no_parameters() {
            let result = self();
            Ok(V::from(result))
        } else {
            Err(
                Error::new(ErrorType::TooManyParameters, format!("Too many parameters"))
                    .with_position(&arguments.action_position),
            )
        }
    }
}

pub struct Command1<S, R, F>
where
    F: FnMut(S) -> R,
{
    f: F,
    state: PhantomData<S>,
    result: PhantomData<R>,
}

impl<S, R, F> From<F> for Command1<S, R, F>
where
    F: FnMut(S) -> R,
{
    fn from(f: F) -> Self {
        Command1 {
            f,
            state: Default::default(),
            result: Default::default(),
        }
    }
}

impl<F, Injection, V, R> Command<Injection, V> for Command1<&State<V>, R, F>
where
    F: FnMut(&State<V>) -> R,
    V: ValueInterface + From<R>,
{
    fn execute(
        &mut self,
        state: &State<V>,
        arguments: &mut CommandArguments<'_, Injection>,
    ) -> Result<V, Error> {
        if arguments.has_no_parameters() {
            let result = (self.f)(state);
            Ok(V::from(result))
        } else {
            Err(
                Error::new(ErrorType::TooManyParameters, format!("Too many parameters"))
                    .with_position(&arguments.action_position),
            )
        }
    }
}

pub struct Command2<S, T, R, F>
where
    F: FnMut(S, T) -> R,
{
    f: F,
    state: PhantomData<S>,
    argument: PhantomData<T>,
    result: PhantomData<R>,
}

impl<S, T, R, F> From<F> for Command2<S, T, R, F>
where
    F: FnMut(S, T) -> R,
{
    fn from(f: F) -> Self {
        Command2 {
            f,
            state: Default::default(),
            result: Default::default(),
            argument: Default::default(),
        }
    }
}

impl<F, Injection, V, T, R> Command<Injection, V> for Command2<&State<V>, T, R, F>
where
    F: FnMut(&State<V>, T) -> R,
    V: ValueInterface + From<R>,
    T: FromParameter<T, Injection>,
{
    fn execute(
        &mut self,
        state: &State<V>,
        arguments: &mut CommandArguments<'_, Injection>,
    ) -> Result<V, Error> {
        if arguments.has_no_parameters() {
            let argument: T = arguments.get()?;
            let result = (self.f)(state, argument);
            Ok(V::from(result))
        } else {
            Err(
                Error::new(ErrorType::TooManyParameters, format!("Too many parameters"))
                    .with_position(&arguments.action_position),
            )
        }
    }
}

pub trait FromParameter<T, Injection> {
    fn from_parameter(param: &Parameter, injection: &Injection) -> Result<T, Error>;
}

impl<I> FromParameter<String, I> for String {
    fn from_parameter(param: &Parameter, _injection: &I) -> Result<String, Error> {
        if let Some(p) = param.value.as_str() {
            Ok(p.to_owned())
        } else {
            //TODO: Use position from parameter
            Err(Error::conversion_error(param.value.clone(), "string"))
        }
    }
}

pub trait CommandExecutor<Injection, V: ValueInterface> {
    fn execute(
        &mut self,
        realm: &str,
        namespace: &str,
        command_name: &str,
        state: &State<V>,
        arguments: &mut CommandArguments<'_, Injection>,
    ) -> Result<V, Error>;
}

impl<I, V: ValueInterface> CommandExecutor<I, V> for HashMap<CommandKey, Box<dyn Command<I, V>>> {
    fn execute(
        &mut self,
        realm: &str,
        namespace: &str,
        command_name: &str,
        state: &State<V>,
        arguments: &mut CommandArguments<I>,
    ) -> Result<V, Error> {
        let key = CommandKey::new(realm, namespace, command_name);
        if let Some(command) = self.get_mut(&key) {
            command.execute(state, arguments)
        } else {
            Err(Error::unknown_command_executor(
                realm,
                namespace,
                command_name,
                &arguments.action_position,
            ))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{state, value::Value};

    struct TestExecutor;
    impl CommandExecutor<NoInjection, Value> for TestExecutor {
        fn execute(
            &mut self,
            realm: &str,
            namespace: &str,
            command_name: &str,
            state: &State<Value>,
            arguments: &mut CommandArguments<'_, NoInjection>,
        ) -> Result<Value, Error> {
            assert_eq!(realm, "");
            assert_eq!(namespace, "");
            assert_eq!(command_name, "test");
            assert!(state.data.is_none());
            (|| -> String { "Hello".into() }).execute(state, arguments)
        }
    }
    #[test]
    fn first_test() {
        let p = Parameter {
            value: "Hello".into(),
            ..Parameter::default()
        };
        let s: String = String::from_parameter(&p, &NoInjection).unwrap();
        assert_eq!(s, "Hello");
    }
    #[test]
    fn test_command_arguments() {
        let mut rp = ResolvedParameters::new();
        rp.parameters.push(Parameter {
            value: "Hello".into(),
            ..Parameter::default()
        });
        let mut ca = CommandArguments::new(rp, &NoInjection);
        let s: String = ca.get().unwrap();
        assert_eq!(s, "Hello");
    }
    #[test]
    fn test_execute_command() -> Result<(), Error> {
        let mut c = || -> String { "Hello".into() };
        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let state: State<Value> = State::new();
        let s: Value = c.execute(&state, &mut ca).unwrap();
        assert_eq!(s.try_into_string()?, "Hello");
        Ok(())
    }

    #[test]
    fn test_command_executor() -> Result<(), Error> {
        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let state = State::new();
        let s = TestExecutor
            .execute("", "", "test", &state, &mut ca)
            .unwrap();
        assert_eq!(s.try_into_string()?, "Hello");
        Ok(())
    }
    #[test]
    fn test_hashmap_command_executor() -> Result<(), Error> {
        let mut hm = HashMap::<CommandKey, Box<dyn Command<NoInjection, Value>>>::new();
        hm.insert(
            CommandKey::new("", "", "test"),
            Box::new(|| -> String { "Hello1".into() }),
        );
        hm.insert(
            CommandKey::new("", "", "test2"),
            Box::new(|| -> String { "Hello2".into() }),
        );

        let state = State::new();
        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let s = hm.execute("", "", "test", &state, &mut ca).unwrap();
        assert_eq!(s.try_into_string()?, "Hello1");
        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let s = hm.execute("", "", "test2", &state, &mut ca).unwrap();
        assert_eq!(s.try_into_string()?, "Hello2");
        Ok(())
    }
}
