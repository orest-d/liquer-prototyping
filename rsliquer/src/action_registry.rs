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

impl<T> CallableAction<T> for Fn(i32)->i32
where
    T:TryInto<i32>,
    i32:TryInto<T>,
    <i32 as std::convert::TryInto<T>>::Error:Display,
    <T as std::convert::TryInto<i32>>::Error:Display
    {
    fn call_action(&self, input:T, arguments:Vec<ActionParameter>) -> Result<T, Error>{
        let f_input:i32 = input.try_into()
        .map_err(|e|
            Error::ConversionError{message:format!("Input argument conversion failed; {}",e)})?;
        let out:i32 = self(f_input);
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
//        let result = a.call_action(Value::Integer(2),vec![])?;
        Ok(())
    }
}