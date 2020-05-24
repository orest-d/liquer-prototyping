use std::result::Result;
use crate::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionParameter{
    String(String),
    Link(String)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ActionRequest{
    pub name:String,
    pub parameters: Vec<ActionParameter>
}

#[derive(Debug)]
pub struct ActionParametersSlice<'a>(pub &'a [ActionParameter]);

pub trait Environment<T>{
    fn eval(&mut self,query:&str)->Result<T,Error>;
}

pub trait TryActionParametersInto<T,E>{
    fn try_parameters_into(&mut self, env:&mut E)->Result<T,Error>;
}

pub trait TryParameterFrom where Self: std::marker::Sized{
    fn try_parameter_from(text:&str)->Result<Self,Error>;
}

impl TryParameterFrom for i32{
    fn try_parameter_from(text:&str)->Result<Self,Error>{
       text.parse().map_err(|e| Error::ParameterError{message:format!("{}",e)})
    }
}

impl TryParameterFrom for String{
    fn try_parameter_from(text:&str)->Result<Self,Error>{
        Ok(text.to_owned())
    }
}

impl<'a,T,E> TryActionParametersInto<T,E>  for ActionParametersSlice<'a> where T:TryParameterFrom{
    fn try_parameters_into(&mut self, env:&mut E)->Result<T,Error>{
        if self.0.is_empty(){
            Err(Error::ArgumentNotSpecified)
        }
        else{
            match &self.0[0]{
                ActionParameter::String(x)=>{
                    let v:T = T::try_parameter_from(&x)?;
                    self.0=&self.0[1..];
                    Ok(v)
                },
                _ =>{
                    Err(Error::General{message:"Not implemented".to_owned()})
                }
            }
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn parameters_into_i32() -> Result<(), Box<dyn std::error::Error>>{
        let v = [ActionParameter::String("123".to_owned()),ActionParameter::String("234".to_owned())];
        let mut par = ActionParametersSlice(&v[..]);
        let x:i32=par.try_parameters_into(&mut ())?;
        assert_eq!(x,123);
        let x:i32=par.try_parameters_into(&mut ())?;
        assert_eq!(x,234);
        Ok(())
    }
    #[test]
    fn parameters_into_str() -> Result<(), Box<dyn std::error::Error>>{
        let v = [ActionParameter::String("123".to_owned()),ActionParameter::String("234".to_owned())];
        let mut par = ActionParametersSlice(&v[..]);
        let x:String=par.try_parameters_into(&mut ())?;
        assert_eq!(x,"123");
        let x:i32=par.try_parameters_into(&mut ())?;
        assert_eq!(x,234);
        Ok(())
    }
}