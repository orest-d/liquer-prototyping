use nom;

extern crate nom_locate;
use nom::branch::alt;
use nom::character::complete::digit1;
use nom_locate::LocatedSpan;


use nom::bytes::complete::{tag, take_while, take_while1};
use nom::*;
use nom::multi::{many0, separated_list0};
use nom::character::{is_alphanumeric, is_alphabetic};
use nom::sequence::pair;

use crate::query::{ActionParameter, ActionRequest, Position, ResourceName};
use crate::error::Error;

type Span<'a> = LocatedSpan<&'a str>;

impl<'a> From<Span<'a>> for Position{
    fn from(span:Span<'a>)->Position{
        Position{
            offset:span.location_offset(),
            line:span.location_line(),
            column:span.get_utf8_column()
        }
    }
}

fn identifier(text:Span) ->IResult<Span, String>{
    let (text, a) =take_while1(|c| {is_alphabetic(c as u8)||c=='_'})(text)?;
    let (text, b) =take_while(|c| {is_alphanumeric(c as u8)||c=='_'})(text)?;

    Ok((text, format!("{}{}",a,b)))
}

fn filename(text:Span) ->IResult<Span, String>{
    let (text, a) =take_while(|c| {is_alphabetic(c as u8)||c=='_'})(text)?;
    let (text, dot) = nom::character::complete::char('.')(text)?;
    let (text, b) =take_while1(|c| {is_alphanumeric(c as u8)||c=='_'||c=='.'||c=='-'})(text)?;

    Ok((text, format!("{}.{}",a,b)))
}

fn resource_name(text:Span)->IResult<Span, ResourceName>{
    let position:Position = text.into();
    let (text, a) =take_while(|c| {is_alphabetic(c as u8)||c=='_'})(text)?;
    let (text, b) =take_while1(|c| {is_alphanumeric(c as u8)||c=='_'||c=='.'||c=='-'})(text)?;
    Ok((text, ResourceName::new(format!("{}{}",a,b)).with_position(position)))
}
fn parameter_text(text:Span)->IResult<Span, String>{
    let (text, a) =take_while1(|c| {is_alphanumeric(c as u8)||c=='_'||c=='+'||c=='.'})(text)?;
    Ok((text, a.to_string()))
}

fn tilde_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~~")(text)?;
    Ok((text, "~".to_owned()))
}
fn minus_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~_")(text)?;
    Ok((text, "-".to_owned()))
}
fn islash_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~I")(text)?;
    Ok((text, "/".to_owned()))
}
fn slash_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~/")(text)?;
    Ok((text, "/".to_owned()))
}
fn https_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~H")(text)?;
    Ok((text, "https://".to_owned()))
}
fn http_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~h")(text)?;
    Ok((text, "http://".to_owned()))
}
fn file_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~f")(text)?;
    Ok((text, "file://".to_owned()))
}
fn protocol_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~P")(text)?;
    Ok((text, "://".to_owned()))
}
fn negative_number_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~")(text)?;
    let (text, n) = digit1(text)?;
    Ok((text, format!("-{n}")))
}
fn space_entity(text:Span) -> IResult<Span, String> {
    let (text, _) = tag("~.")(text)?;
    Ok((text, " ".to_owned()))
}
fn entities(text:Span) -> IResult<Span, String> {
    alt((
        tilde_entity,
        minus_entity,
        negative_number_entity,
        space_entity,
        islash_entity,
        slash_entity,
        http_entity,
        https_entity,
        file_entity,
        protocol_entity    
    ))(text)
}
fn parameter(text:Span) ->IResult<Span, ActionParameter>{
    let position:Position = text.into();
    let (text, par) =many0(alt((parameter_text, entities)))(text)?;
    Ok((text, ActionParameter::new_string(par.join("")).with_position(position)))
}
fn minus_parameter(text:Span) ->IResult<Span, ActionParameter>{
    let (text, _) =tag("-")(text)?;
    parameter(text)
}
    /*
fn parameter(text:Span) ->IResult<Span, ActionParameter>{
    let position:Position = text.into();
    let (text, par) =take_while(|c| {c!='-'&&c!='/'})(text)?;

    Ok((text, ActionParameter::new_string(par.to_string()).with_position(position)))
}
*/
fn action_request(text:Span) ->IResult<Span, ActionRequest>{
    let position:Position = text.into();
    let (text, name) = identifier(text)?;
    let (text, parameters) =many0(minus_parameter)(text)?;
    Ok((text, ActionRequest::new(name).with_parameters(parameters).with_position(position)))
}



/*
fn parse_action(text:Span) ->IResult<Span, ActionRequest>{
    let position:Position = text.into();
    let (text, name) =identifier(text)?;
    let (text, p) =many0(pair(tag("-"),parameter))(text)?;

    Ok((text, ActionRequest{name:name, position, parameters:p.iter().map(|x| x.1.clone()).collect()}))
}
*/

fn parse_action_path(text:Span) -> IResult<Span, Vec<ActionRequest>>{
    separated_list0(tag("/"), action_request)(text)
}

pub fn parse_query(query:&str)-> Result<Vec<ActionRequest>, Error>{
    let (remainder, path)  = parse_action_path(Span::new(query)).map_err(|e| Error::General{message:format!("Parse error {}",e)})?;
    if remainder.fragment().len()>0{
        Err(Error::ParseError{message:format!("Can't parse query completely: '{}'",remainder.fragment()), position:remainder.into()})
    }
    else{
        Ok(path)
    }
}


#[cfg(test)]
mod tests{
    use super::*;
    use crate::query::{ActionParameter};

    #[test]
    fn parse_action_test() -> Result<(), Box<dyn std::error::Error>>{
        let (_remainder, action)  = action_request(Span::new("abc-def"))?;
        assert_eq!(action.name,"abc");
        assert_eq!(action.parameters.len(),1);
        match &action.parameters[0]{
            ActionParameter::String(txt,_) => assert_eq!(txt, "def"),
            _ => assert!(false)
        }
        Ok(())
    }
    #[test]
    fn parse_path_test() -> Result<(), Box<dyn std::error::Error>>{
        let (remainder, path)  = parse_action_path(Span::new("abc-def/xxx-123"))?;
        println!("REMAINDER: {:#?}",remainder);
        println!("PATH:      {:#?}",path);
        assert_eq!(remainder.fragment().len(),0);
        assert_eq!(remainder.to_string().len(),0);
        Ok(())
    }
    #[test]
    fn parse_query_test() -> Result<(), Error>{
        let path  = parse_query("")?;
        assert_eq!(path.len(),0);
        let path  = parse_query("abc-def")?;
        assert_eq!(path.len(),1);
        let path  = parse_query("abc-def/xxx-123")?;
        assert_eq!(path.len(),2);
        Ok(())
    }

}