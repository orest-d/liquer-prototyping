use std::sync::Arc;
use crate::error::Error;

use crate::metadata::Metadata;
use crate::query::{ActionRequest, ActionParameter, Position};
use crate::value::ValueInterface;

pub struct CommandParameter<V> where V:ValueInterface {
    value:Arc<V>,
    position:Position
}


pub trait Context{
    
}
struct DummyContext;
impl Context for DummyContext{

}

pub trait Command<V> where V:ValueInterface {
    fn call_command(&mut self, state_data:Arc<V>, state_metadata:Arc<Metadata>, parameters:&[CommandParameter<V>], context:impl Context) -> Result<V, Error>;
    fn execute_action(&mut self, state_data:Arc<V>, state_metadata:Arc<Metadata>, action:&ActionRequest, context: impl Context) -> Result<V, Error>{
        let mut par = Vec::with_capacity(action.parameters.len());
        for (i, p) in action.parameters.iter().enumerate() {
            match p{
                ActionParameter::String(v, pos) => par.push(CommandParameter{value:Arc::new(V::new(&v)), position:pos.clone()}),
                ActionParameter::Link(_, _) => todo!()
            }
        }
        self.call_command(state_data, state_metadata, par.as_slice(), context)
    }
}

impl<F,V> Command<V> for F where F:FnMut()->(), V:ValueInterface{
    fn call_command(&mut self, state_data:Arc<V>, state_metadata:Arc<Metadata>, parameters:&[CommandParameter<V>], context:impl Context) -> Result<V, Error>{
        if parameters.is_empty() {
            self();
            Ok(V::none())
        }
        else{
            Err(Error::ParameterError{message:format!("Too many parameters"), position:Position::unknown()})
        }
    }
}

#[cfg(test)]
mod test{
    use crate::value::Value;

    use super::*;

    #[test]
    fn test_command()->Result<(),Box<dyn std::error::Error>>{
        let q = crate::parse::parse_query("abc")?;
        let a = q.action().unwrap();
        let mut called = false;
        assert!(!called);
        let mut f =||{called=true;};
        let mut context = DummyContext;
        (&mut f).execute_action(Arc::new(Value::none()), Arc::new(Metadata::new()), &a, context);
        assert!(called);
        Ok(())
    }
}