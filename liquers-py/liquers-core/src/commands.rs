use std::collections::HashMap;
use std::result;

use crate::command_metadata::{CommandMetadata, CommandKey};
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
    fn execute(&mut self, arguments: CommandArguments<Injection>) -> Result<State<V>, Error>;

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
    fn execute(&mut self, arguments: CommandArguments<Injection>) -> Result<State<V>, Error> {
        if arguments.has_no_parameters() {
            let result = self();
            Ok(State::new().with_data(V::from(result)))
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
        arguments: CommandArguments<Injection>,
    ) -> Result<State<V>, Error>;
}

impl<I,V:ValueInterface> CommandExecutor<I, V> for HashMap<CommandKey, Box<dyn Command<I, V>>> {
    fn execute(
        &mut self,
        realm: &str,
        namespace: &str,
        command_name: &str,
        arguments: CommandArguments<I>,
    ) -> Result<State<V>, Error> {
        let key = CommandKey::new(realm, namespace, command_name);
        if let Some(command) = self.get_mut(&key) {
            command.execute(arguments)
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
    use crate::value::Value;

    struct TestExecutor;
    impl CommandExecutor<NoInjection,Value> for TestExecutor {
        fn execute(
            &mut self,
            realm: &str,
            namespace: &str,
            command_name: &str,
            arguments: CommandArguments<NoInjection>,
        ) -> Result<State<Value>, Error> {
            assert_eq!(realm,"");
            assert_eq!(namespace,"");
            assert_eq!(command_name,"test");
            (|| -> String { "Hello".into() }).execute(arguments)
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
        let s: State<Value> = c.execute(ca).unwrap();
        assert_eq!(s.data.try_into_string()?, "Hello");
        Ok(())
    }

    #[test]
    fn test_command_executor()->Result<(),Error>{
        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let s: State<Value> = TestExecutor.execute("", "", "test", ca).unwrap();
        assert_eq!(s.data.try_into_string()?, "Hello");
        Ok(())
    }
    #[test]
    fn test_hashmap_command_executor()->Result<(),Error>{
        let mut hm = HashMap::<CommandKey, Box<dyn Command<NoInjection, Value>>>::new();
        hm.insert(CommandKey::new("", "", "test"), Box::new(|| -> String { "Hello1".into() }));
        hm.insert(CommandKey::new("", "", "test2"), Box::new(|| -> String { "Hello2".into() }));

        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let s: State<Value> = hm.execute("", "", "test", ca).unwrap();
        assert_eq!(s.data.try_into_string()?, "Hello1");
        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let s: State<Value> = hm.execute("", "", "test2", ca).unwrap();
        assert_eq!(s.data.try_into_string()?, "Hello2");
        Ok(())
    }
}
