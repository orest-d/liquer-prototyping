#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::marker::PhantomData;

use crate::command_metadata::{self, CommandKey, CommandMetadata, CommandMetadataRegistry};
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
    pub fn len(&self) -> usize {
        self.parameters.parameters.len()
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

/*
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
                Error::new(ErrorType::TooManyParameters, format!("Too many parameters ({}); none expected", arguments.len()))
                    .with_position(&arguments.action_position),
            )
        }
    }
}
*/

pub struct Command0<R, F>
where
    F: FnMut() -> R,
{
    f: F,
    result: PhantomData<R>,
}

impl<R, F> From<F> for Command0<R, F>
where
    F: FnMut() -> R,
{
    fn from(f: F) -> Self {
        Command0 {
            f,
            result: Default::default(),
        }
    }
}

impl<F, Injection, V, R> Command<Injection, V> for Command0<R, F>
where
    F: FnMut() -> R,
    V: ValueInterface + From<R>,
{
    fn execute(
        &mut self,
        state: &State<V>,
        arguments: &mut CommandArguments<'_, Injection>,
    ) -> Result<V, Error> {
        if arguments.has_no_parameters() {
            let result = (self.f)();
            Ok(V::from(result))
        } else {
            Err(
                Error::new(ErrorType::TooManyParameters, format!("Too many parameters ({}) - none expected", arguments.len()))
                    .with_position(&arguments.action_position),
            )
        }
    }
}
/*
impl<F: FnMut() -> R + 'static, Injection, V, R> From<F> for Box<dyn Command<Injection, V>>
where
R: 'static,
V: ValueInterface + From<R> + 'static,
{
    fn from(f: F) -> Self {
        Box::new(Command0::from(f))
    }
}
*/

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
                Error::new(ErrorType::TooManyParameters, format!("Too many parameters ({}) - only state expected", arguments.len()))
                    .with_position(&arguments.action_position),
            )
        }
    }
}
/*
impl<F: FnMut(&State<V>) -> R + 'static, Injection, V, R> From<F> for Box<dyn Command<Injection, V>>
where
R: 'static,
V: ValueInterface + From<R> + 'static,
{
    fn from(f: F) -> Self {
        Box::new(Command1::from(f))
    }
}
*/
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
        if arguments.len()==1 {
            let argument: T = arguments.get()?;
            let result = (self.f)(state, argument);
            Ok(V::from(result))
        } else {
            Err(
                Error::new(ErrorType::TooManyParameters, format!("Too many parameters ({}), 1 expected", arguments.len()))
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
/*
impl<I> FromParameter<&str, I> for &str {
    fn from_parameter(param: &Parameter, _injection: &I) -> Result<&str, Error> {
        if let Some(p) = param.value.as_str() {
            Ok(p)
        } else {
            //TODO: Use position from parameter
            Err(Error::conversion_error(param.value.clone(), "string (&str)"))
        }
    }
}
*/

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
pub struct CommandRegistry<I, V: ValueInterface> {
    executors: HashMap<CommandKey, Box<dyn Command<I, V>>>,
    pub command_metadata_registry: CommandMetadataRegistry,
}

impl<I, V: ValueInterface> CommandRegistry<I, V> {
    pub fn new() -> Self {
        CommandRegistry {
            executors: HashMap::new(),
            command_metadata_registry: CommandMetadataRegistry::new(),
        }
    }
    pub fn register_boxed_command<K>(
        &mut self,
        key: K,
        executor: Box<dyn Command<I, V>>,
    ) -> Result<&mut CommandMetadata, Error>
    where
        K: Into<CommandKey>,
    {
        let key = key.into();
        let mut command_metadata = executor
            .command_metadata()
            .map(|cm| {
                let mut cm = cm.clone();
                cm.with_realm(&key.realm)
                    .with_namespace(&key.namespace)
                    .with_name(&key.name);
                cm
            })
            .unwrap_or((&key).into());
        self.command_metadata_registry
            .add_command(&command_metadata);

        self.executors.insert(key.clone(), executor);
        Ok(self.command_metadata_registry.get_mut(key).unwrap())
    }
    pub fn register_command<K, T>(&mut self, key: K, f: T) -> Result<&mut CommandMetadata, Error>
    where
        K: Into<CommandKey>,
        T: Command<I, V> + 'static,
    {
        let key = key.into();
        let command: Box<dyn Command<I, V>> = Box::new(f);
        self.register_boxed_command(key, command)
    }
    /*
    pub fn register<K, T>(&mut self, key: K, f: T) -> Result<&mut CommandMetadata, Error>
    where
        K: Into<CommandKey>,
        T: Into<Box<dyn Command<I, V>>> + 'static,
    {
        let key = key.into();
        let command: Box<dyn Command<I, V>> = f.into();
        self.register_boxed_command(key, command)
    }
    */
}

impl<I, V: ValueInterface> CommandExecutor<I, V> for CommandRegistry<I, V> {
    fn execute(
        &mut self,
        realm: &str,
        namespace: &str,
        command_name: &str,
        state: &State<V>,
        arguments: &mut CommandArguments<I>,
    ) -> Result<V, Error> {
        let key = CommandKey::new(realm, namespace, command_name);
        if let Some(command) = self.executors.get_mut(&key) {
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
            Command0::from(|| -> String { "Hello".into() }).execute(state, arguments)
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
        let mut c = Command0::from(|| -> String { "Hello".into() });
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
            Box::new(Command0::from(|| -> String { "Hello1".into() })),
        );
        hm.insert(
            CommandKey::new("", "", "test2"),
            Box::new(Command0::from(|| -> String { "Hello2".into() })),
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
    #[test]
    fn test_command_registry() -> Result<(), Error> {
        let mut cr = CommandRegistry::<NoInjection, Value>::new();
        cr.register_command("test", Command0::from(|| -> String { "Hello1".into() }))?;
        cr.register_command("test2", Command0::from(|| -> String { "Hello2".into() }))?;
        cr.register_command("stest1", Command1::from(|s:&State<Value>| -> String { "STest1".into() }))?;
        println!("{:?}", cr.command_metadata_registry);

        let state = State::new();
        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let s = cr.execute("", "", "test", &state, &mut ca).unwrap();
        assert_eq!(s.try_into_string()?, "Hello1");
        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let s = cr.execute("", "", "test2", &state, &mut ca).unwrap();
        assert_eq!(s.try_into_string()?, "Hello2");
        Ok(())
    }

}
