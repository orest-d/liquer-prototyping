use std::result;

use crate::command_metadata::CommandMetadata;
use crate::error::Error;
use crate::plan::{ResolvedParameters, Parameter};
use crate::query::Position;
use crate::state::State;
use crate::value::ValueInterface;

pub struct NoInjection;


pub struct CommandArguments<'i, Injection>{
    pub parameters: ResolvedParameters,
    pub injection: &'i Injection,
    pub action_position: Position,
    pub argument_number: usize,
}

impl<'i, Injection> CommandArguments<'i, Injection>{
    pub fn new(parameters: ResolvedParameters, injection: &'i Injection) -> Self{
        CommandArguments{
            parameters,
            injection,
            action_position: Position::unknown(),
            argument_number: 0,
        }
    }
    pub fn has_no_parameters(&self)->bool{
        self.parameters.parameters.is_empty()
    }
    pub fn get<T: FromParameter<T, Injection>>(&mut self)->Result<T, Error>{
        if let Some(p) = self.parameters.parameters.get(self.argument_number){
            self.argument_number += 1;
            T::from_parameter(p, self.injection)
        }
        else{
            Err(Error::missing_argument(self.argument_number, "?", &self.action_position))
        }
    }
}

/// Command trait
/// This trait encapsulates a command that can be executed,
/// typically a function
pub trait Command<Injection, V: ValueInterface> {
    fn execute(
        &mut self,
        arguments: CommandArguments<Injection>,
    ) -> Result<State<V>, Error>;

    /// Returns the default metadata of the command
    /// This may be modified or overriden inside the command registry
    fn command_metadata(&self) -> Option<CommandMetadata>{
        None
    }
}

impl<F, R, Injection, V> Command<Injection, V> for F
where
    F: FnMut() -> R,
    V: ValueInterface + From<R>
{
    fn execute(
        &mut self,
        arguments: CommandArguments<Injection>,
    ) -> Result<State<V>, Error> {
        if arguments.has_no_parameters() {
            let result = self();
            Ok(State::new().with_data(V::from(result)))
        } else {
            Err(Error::ParameterError {
                message: format!("Too many parameters"),
                position: arguments.action_position.clone(),
            })
        }
    }
}

pub trait FromParameter<T,Injection>{
    fn from_parameter(param: &Parameter, injection:&Injection) -> Result<T, Error>;
}

impl<I> FromParameter<String, I> for String {
    fn from_parameter(param: &Parameter, _injection:&I) -> Result<String, Error> {
        if let Some(p) = param.0.as_str(){
            Ok(p.to_owned())
        }
        else{
            //TODO: Use position from parameter
            Err(Error::conversion_error(param.0.clone(), "string"))
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::value::Value;

    #[test]
    fn first_test() {
        let p = Parameter("Hello".into());
        let s:String = String::from_parameter(&p, &NoInjection).unwrap();
        assert_eq!(s, "Hello");
    }
    #[test]
    fn test_command_arguments(){
        let mut rp = ResolvedParameters::new();
        rp.parameters.push(Parameter("Hello".into()));
        let mut ca = CommandArguments::new(rp, &NoInjection);
        let s:String = ca.get().unwrap();
        assert_eq!(s, "Hello");
    }
    #[test]
    fn test_execute_command()->Result<(), Error>{
        let mut c = || -> String{
            "Hello".into()
        };
        let mut ca = CommandArguments::new(ResolvedParameters::new(), &NoInjection);
        let s:State<Value> = c.execute(ca).unwrap();
        assert_eq!(s.data.try_into_string()?, "Hello");
        Ok(())
    }

}