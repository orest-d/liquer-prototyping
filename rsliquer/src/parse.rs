use nom;

use nom::bytes::complete::{tag, take_while, take_while1};
use nom::*;
use nom::multi::{many0, separated_list};
use nom::character::{is_alphanumeric, is_alphabetic};
use nom::sequence::pair;

use crate::query::{ActionParameter, ActionRequest};

fn identifier(text:&str) ->IResult<&str, String>{
    let (text, a) =take_while1(|c| {is_alphabetic(c as u8)||c=='_'})(text)?;
    let (text, b) =take_while(|c| {is_alphanumeric(c as u8)||c=='_'})(text)?;

    Ok((text, format!("{}{}",a,b)))
}
fn parameter(text:&str) ->IResult<&str, String>{
    let (text, par) =take_while(|c| {c!='-'&&c!='/'})(text)?;

    Ok((text, par.to_owned()))
}


fn parse_action(text:&str) ->IResult<&str, ActionRequest>{
    let (text, name) =identifier(text)?;
    let (text, p) =many0(pair(tag("-"),parameter))(text)?;

    Ok((text, ActionRequest{name:name, parameters:p.iter().map(|x| ActionParameter::String(x.1.to_owned())).collect()}))
}

fn parse_action_path(text:&str) ->IResult<&str, Vec<ActionRequest>>{
    separated_list(tag("/"), parse_action)(text)
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::query::{ActionParameter};

    #[test]
    fn parse_action_test() -> Result<(), Box<dyn std::error::Error>>{
        let (_remainder, action)  = parse_action("abc-def")?;
        assert_eq!(action.name,"abc");
        assert_eq!(action.parameters.len(),1);
        match &action.parameters[0]{
            ActionParameter::String(txt) => assert_eq!(txt, "def"),
            _ => assert!(false)
        }
        Ok(())
    }
}