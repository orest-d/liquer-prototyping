use std::result::Result;

use crate::error::Error;
use crate::query::ActionParameter;
use std::convert::TryInto;
use core::fmt::Display;
use std::ops::Fn;

trait CallableAction<T>{
    fn call_action(&self, input:T, arguments:Vec<ActionParameter>) -> Result<T, Error>;
}
/*
impl<T,In,Out> CallableAction<T> for Fn(In)->Out
where T:TryInto<In>, Out:TryInto<T>{
    pub fn call(&self, input:T, arguments:Vec<ActionParameter>) -> Result<T, Error>{
        let f_input:In = input.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Input argument conversion failed; {}",e)})?
        let out = self(f_input);
        let result:T = out.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Result conversion failed; {}",e)})?
        Ok(result)
    }
}
*/

//pub struct Function1<In,Out>(Box<dyn FnMut(In) -> Out>);

impl<F,T> CallableAction<T> for F
where
    F:Fn(i32)->i32,
    T:TryInto<i32>,
    i32:TryInto<T>,
    <i32 as std::convert::TryInto<T>>::Error:Display,
    <T as std::convert::TryInto<i32>>::Error:Display
    {
    fn call_action(&self, input:T, _arguments:Vec<ActionParameter>) -> Result<T, Error>{
        let f_input:i32 = input.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Input argument conversion failed; {}",e)})?;

            let out:i32 = (*self)(f_input);
            let result:T = out.try_into()
            .map_err(|e|
                Error::ConversionError{message:format!("Result conversion failed; {}",e)})?;
                Ok(result)
    }
}

pub struct Function1<In,Out>(Box<dyn Fn(In)->Out>);

impl<T> CallableAction<T> for Function1<i32,i32>
where
    T:TryInto<i32>,
    i32:TryInto<T>,
    <i32 as std::convert::TryInto<T>>::Error:Display,
    <T as std::convert::TryInto<i32>>::Error:Display
    {
    fn call_action(&self, input:T, _arguments:Vec<ActionParameter>) -> Result<T, Error>{
        let f_input:i32 = input.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Input argument conversion failed; {}",e)})?;

            let out:i32 = self.0(f_input);
            let result:T = out.try_into()
            .map_err(|e|
                Error::ConversionError{message:format!("Result conversion failed; {}",e)})?;
                Ok(result)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::value::*;

    #[test]
    fn test1()-> Result<(), Box<dyn std::error::Error>>{
        let a = |x:i32| x*x;
        let result = a.call_action(Value::Integer(2),vec![])?;
        assert_eq!(result, Value::Integer(4));
        Ok(())
    }

    #[test]
    fn test2()-> Result<(), Box<dyn std::error::Error>>{
        let a = |x:i32| x*x;
        let f:Function1<i32,i32> = Function1(Box::new(a));
        let result = f.call_action(Value::Integer(2),vec![])?;
        assert_eq!(result, Value::Integer(4));
        Ok(())
    }
}