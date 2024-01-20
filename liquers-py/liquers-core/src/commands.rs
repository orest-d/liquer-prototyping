use crate::command_registry::CommandMetadata;
use crate::error::Error;
use crate::plan::{ResolvedParameters, Parameter};
use crate::query::Position;
use crate::state::State;
use crate::value::ValueInterface;

pub struct NoInjection;

/// Command trait
/// This trait encapsulates a command that can be executed,
/// typically a function
pub trait Command<Injection, V: ValueInterface> {
    fn execute(
        &self,
        params: ResolvedParameters,
        position: &Position,
        injection: &Injection,
    ) -> Result<State<V>, Error>;

    /// Returns the default metadata of the command
    /// This may be modified or overriden inside the command registry
    fn command_metadata(&self) -> Option<CommandMetadata>{
        None
    }
}

//TODO: Use position from parameter
pub trait FromParameter<T,Injection>{
    fn from_parameter(param: &Parameter, injection:&Injection, position: &Position) -> Result<T, Error>;
}

impl<I> FromParameter<String, I> for String {
    fn from_parameter(param: &Parameter, _injection:&I, position: &Position) -> Result<String, Error> {
        if let Some(p) = param.0.as_str(){
            Ok(p.to_owned())
        }
        else{
            Err(Error::conversion_error_at_position(param.0.clone(), "string", position))        
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn first_test() {
        let p = Parameter("Hello".into());
        let s:String = String::from_parameter(&p, &NoInjection, &Position::unknown()).unwrap();
        assert_eq!(s, "Hello");
    }

}