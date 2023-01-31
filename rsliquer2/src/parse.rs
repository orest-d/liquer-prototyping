use nom;

extern crate nom_locate;
use nom::branch::alt;
use nom::character::complete::digit1;
use nom::combinator::opt;
use nom_locate::LocatedSpan;

use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::{is_alphabetic, is_alphanumeric};
use nom::multi::{many0, many1, separated_list0, separated_list1};
use nom::sequence::pair;
use nom::*;

use crate::error::Error;
use crate::query::{
    ActionParameter, ActionRequest, HeaderParameter, Position, Query, QuerySegment, ResourceName,
    ResourceQuerySegment, SegmentHeader, TransformQuerySegment,
};

type Span<'a> = LocatedSpan<&'a str>;

impl<'a> From<Span<'a>> for Position {
    fn from(span: Span<'a>) -> Position {
        Position {
            offset: span.location_offset(),
            line: span.location_line(),
            column: span.get_utf8_column(),
        }
    }
}

fn identifier(text: Span) -> IResult<Span, String> {
    let (text, a) = take_while1(|c| is_alphabetic(c as u8) || c == '_')(text)?;
    let (text, b) = take_while(|c| is_alphanumeric(c as u8) || c == '_')(text)?;

    Ok((text, format!("{}{}", a, b)))
}

fn filename(text: Span) -> IResult<Span, String> {
    let (text, a) = take_while(|c| is_alphabetic(c as u8) || c == '_')(text)?;
    let (text, dot) = nom::character::complete::char('.')(text)?;
    let (text, b) =
        take_while1(|c| is_alphanumeric(c as u8) || c == '_' || c == '.' || c == '-')(text)?;

    Ok((text, format!("{}.{}", a, b)))
}

fn resource_name(text: Span) -> IResult<Span, ResourceName> {
    let position: Position = text.into();
    let (text, a) = take_while(|c| is_alphabetic(c as u8) || c == '_')(text)?;
    let (text, b) =
        take_while1(|c| is_alphanumeric(c as u8) || c == '_' || c == '.' || c == '-')(text)?;
    Ok((
        text,
        ResourceName::new(format!("{}{}", a, b)).with_position(position),
    ))
}
fn parameter_text(text: Span) -> IResult<Span, String> {
    let (text, a) =
        take_while1(|c| is_alphanumeric(c as u8) || c == '_' || c == '+' || c == '.')(text)?;
    Ok((text, a.to_string()))
}

fn tilde_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~~")(text)?;
    Ok((text, "~".to_owned()))
}
fn minus_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~_")(text)?;
    Ok((text, "-".to_owned()))
}
fn islash_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~I")(text)?;
    Ok((text, "/".to_owned()))
}
fn slash_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~/")(text)?;
    Ok((text, "/".to_owned()))
}
fn https_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~H")(text)?;
    Ok((text, "https://".to_owned()))
}
fn http_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~h")(text)?;
    Ok((text, "http://".to_owned()))
}
fn file_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~f")(text)?;
    Ok((text, "file://".to_owned()))
}
fn protocol_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~P")(text)?;
    Ok((text, "://".to_owned()))
}
fn negative_number_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~")(text)?;
    let (text, n) = digit1(text)?;
    Ok((text, format!("-{n}")))
}
fn space_entity(text: Span) -> IResult<Span, String> {
    let (text, _) = tag("~.")(text)?;
    Ok((text, " ".to_owned()))
}
fn entities(text: Span) -> IResult<Span, String> {
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
        protocol_entity,
    ))(text)
}
fn parameter(text: Span) -> IResult<Span, ActionParameter> {
    let position: Position = text.into();
    let (text, par) = many0(alt((parameter_text, entities)))(text)?;
    Ok((
        text,
        ActionParameter::new_string(par.join("")).with_position(position),
    ))
}
fn minus_parameter(text: Span) -> IResult<Span, ActionParameter> {
    let (text, _) = tag("-")(text)?;
    parameter(text)
}
/*
fn parameter(text:Span) ->IResult<Span, ActionParameter>{
    let position:Position = text.into();
    let (text, par) =take_while(|c| {c!='-'&&c!='/'})(text)?;

    Ok((text, ActionParameter::new_string(par.to_string()).with_position(position)))
}
*/
fn action_request(text: Span) -> IResult<Span, ActionRequest> {
    let position: Position = text.into();
    let (text, name) = identifier(text)?;
    let (text, parameters) = many0(minus_parameter)(text)?;
    Ok((
        text,
        ActionRequest::new(name)
            .with_parameters(parameters)
            .with_position(position),
    ))
}

fn header_parameter(text: Span) -> IResult<Span, HeaderParameter> {
    let (text, _) = tag("-")(text)?;
    let position: Position = text.into();
    let (text, parameter) = take_while(|c| is_alphanumeric(c as u8) || c == '_' || c == '.')(text)?;
    Ok((
        text,
        HeaderParameter::new(parameter.to_string()).with_position(position),
    ))
}

fn full_transform_segment_header(text: Span) -> IResult<Span, SegmentHeader> {
    let position: Position = text.into();
    let (text, level_lead) = many1(tag("-"))(text)?;
    let (text, lead_name) =
        take_while1(|c: char| is_alphabetic(c as u8) && c.is_lowercase())(text)?;
    let (text, rest_name) = take_while(|c| is_alphanumeric(c as u8) || c == '_')(text)?;
    let (text, parameters) = many0(header_parameter)(text)?;
    let (text, _) = tag("/")(text)?;

    Ok((
        text,
        SegmentHeader {
            name: format!("{lead_name}{rest_name}"),
            level: level_lead.len() - 1,
            parameters,
            resource: false,
            position,
        },
    ))
}

fn short_transform_segment_header(text: Span) -> IResult<Span, SegmentHeader> {
    let position: Position = text.into();
    let (text, level_lead) = many1(tag("-"))(text)?;
    let (text, _) = tag("/")(text)?;

    Ok((
        text,
        SegmentHeader {
            name: "".to_owned(),
            level: level_lead.len() - 1,
            parameters: vec![],
            resource: false,
            position,
        },
    ))
}

fn transform_segment_header(text: Span) -> IResult<Span, SegmentHeader> {
    alt((
        short_transform_segment_header,
        full_transform_segment_header,
    ))(text)
}

fn resource_segment_header(text: Span) -> IResult<Span, SegmentHeader> {
    let position: Position = text.into();
    let (text, level_lead) = many1(tag("-"))(text)?;
    let (text, _) = tag("R")(text)?;
    let (text, name) = take_while(|c: char| is_alphanumeric(c as u8) || c == '_')(text)?;
    let (text, parameters) = many0(header_parameter)(text)?;
    let (text, _) = tag("/")(text)?;

    Ok((
        text,
        SegmentHeader {
            name: name.to_string(),
            level: level_lead.len() - 1,
            parameters,
            resource: true,
            position,
        },
    ))
}

fn resource_path(text: Span) -> IResult<Span, Vec<ResourceName>> {
    separated_list0(tag("/"), resource_name)(text)
}
fn resource_path1(text: Span) -> IResult<Span, Vec<ResourceName>> {
    separated_list1(tag("/"), resource_name)(text)
}

fn resource_segment_with_header(text: Span) -> IResult<Span, ResourceQuerySegment> {
    let (text, header) = resource_segment_header(text)?;
    let (text, query) = resource_path(text)?;
    Ok((
        text,
        ResourceQuerySegment {
            header: Some(header),
            query,
        },
    ))
}
fn resource_qs(text: Span) -> IResult<Span, QuerySegment> {
    let (text, rqs) = resource_segment_with_header(text)?;
    Ok((text, QuerySegment::Resource(rqs)))
}

fn transform_segment_with_header(text: Span) -> IResult<Span, TransformQuerySegment> {
    let (text, header) = transform_segment_header(text)?;
    let (text, query) = separated_list1(tag("/"), action_request)(text)?;
    let position: Position = text.into();
    let (text, fname) = opt(filename)(text)?;
    Ok((
        text,
        TransformQuerySegment {
            header: Some(header),
            query,
            filename: fname.map(|name| ResourceName::new(name).with_position(position)),
        },
    ))
}
fn transform_qs(text: Span) -> IResult<Span, QuerySegment> {
    let (text, tqs) = transform_segment_with_header(text)?;
    Ok((text, QuerySegment::Transform(tqs)))
}
fn query_segment(text: Span) -> IResult<Span, QuerySegment> {
    alt((resource_qs, transform_qs))(text)
}

fn transform_segment_without_header(text: Span) -> IResult<Span, TransformQuerySegment> {
    let (text, query) = separated_list1(tag("/"), action_request)(text)?;
    let position: Position = text.into();
    let (text, fname) = opt(filename)(text)?;
    Ok((
        text,
        TransformQuerySegment {
            header: None,
            query,
            filename: fname.map(|name| ResourceName::new(name).with_position(position)),
        },
    ))
}

fn simple_transform_query(text: Span) -> IResult<Span, Query> {
    let (text, abs) = opt(tag("/"))(text)?;
    let (text, tqs) = transform_segment_without_header(text)?;
    Ok((
        text,
        Query {
            segments: vec![QuerySegment::Transform(tqs)],
            absolute: abs.is_some(),
        },
    ))
}

fn resource_transform_query(text: Span) -> IResult<Span, Query> {
    let (text, abs) = opt(tag("/"))(text)?;
    let (text, resource) = resource_path1(text)?;
    let (text, tqs) = transform_segment_with_header(text)?;
    Ok((
        text,
        Query {
            segments: vec![
                QuerySegment::Resource(ResourceQuerySegment {
                    header: None,
                    query: resource,
                }),
                QuerySegment::Transform(tqs),
            ],
            absolute: abs.is_some(),
        },
    ))
}
fn general_query(text: Span) -> IResult<Span, Query> {
    let (text, abs) = opt(tag("/"))(text)?;
    let (text, segments) = many1(query_segment)(text)?;
    Ok((
        text,
        Query {
            segments,
            absolute: abs.is_some(),
        },
    ))
}

fn empty_query(text: Span) -> IResult<Span, Query> {
    let (text, abs) = opt(tag("/"))(text)?;
    Ok((
        text,
        Query {
            segments:vec![],
            absolute: abs.is_some(),
        },
    ))
}

fn query_parser(text: Span) -> IResult<Span, Query> {
    alt((simple_transform_query, resource_transform_query, general_query, empty_query))(text)
}
/*
fn parse_action(text:Span) ->IResult<Span, ActionRequest>{
    let position:Position = text.into();
    let (text, name) =identifier(text)?;
    let (text, p) =many0(pair(tag("-"),parameter))(text)?;

    Ok((text, ActionRequest{name:name, position, parameters:p.iter().map(|x| x.1.clone()).collect()}))
}

fn parse_action_path(text: Span) -> IResult<Span, Vec<ActionRequest>> {
    separated_list0(tag("/"), action_request)(text)
}
*/

pub fn parse_query(query: &str) -> Result<Query, Error> {
    let (remainder, path) = query_parser(Span::new(query)).map_err(|e| Error::General {
        message: format!("Parse error {}", e),
    })?;
    if remainder.fragment().len() > 0 {
        Err(Error::ParseError {
            message: format!("Can't parse query completely: '{}'", remainder.fragment()),
            position: remainder.into(),
        })
    } else {
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::ActionParameter;

    #[test]
    fn parse_action_test() -> Result<(), Box<dyn std::error::Error>> {
        let (_remainder, action) = action_request(Span::new("abc-def"))?;
        assert_eq!(action.name, "abc");
        assert_eq!(action.parameters.len(), 1);
        match &action.parameters[0] {
            ActionParameter::String(txt, _) => assert_eq!(txt, "def"),
            _ => assert!(false),
        }
        Ok(())
    }
    #[test]
    fn parse_path_test() -> Result<(), Box<dyn std::error::Error>> {
        let (remainder, path) = query_parser(Span::new("abc-def/xxx-123"))?;
        println!("REMAINDER: {:#?}", remainder);
        println!("PATH:      {:#?}", path);
        assert_eq!(remainder.fragment().len(), 0);
        assert_eq!(remainder.to_string().len(), 0);
        Ok(())
    }
    #[test]
    fn parse_query_test() -> Result<(), Error> {
        let path = parse_query("")?;
        assert_eq!(path.len(), 0);
        let path = parse_query("abc-def")?;
        assert_eq!(path.len(), 1);
        let path = parse_query("abc-def/xxx-123")?;
        assert_eq!(path.len(), 1);
        
        assert_eq!(path.segments[0].len(), 2);
        Ok(())
    }
    #[test]
    fn parse_ns() -> Result<(), Error> {
        let path = parse_query("ns-abc")?;
        assert!(path.is_ns());
        assert_eq!(path.ns().unwrap().len(),1);
        assert_eq!(path.ns().unwrap()[0].encode(),"abc");
        Ok(())
    }
    #[test]
    fn parse_last_ns() -> Result<(), Error> {
        let path = parse_query("ns-abc/test")?;
        assert!(!path.is_ns());
        assert_eq!(path.last_ns().unwrap().len(),1);
        assert_eq!(path.last_ns().unwrap()[0].encode(),"abc");
        let path = parse_query("test")?;
        assert!(!path.is_ns());
        assert!(path.last_ns().is_none());
        Ok(())
    }
}
