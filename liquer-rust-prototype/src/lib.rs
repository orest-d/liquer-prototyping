#[macro_use]
extern crate nom;
extern crate regex;

extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

#[macro_use]
extern crate serde_derive;


use nom::bytes::complete::{tag, is_a, take_while, take_while1};
use nom::*;
use nom::character::complete::alphanumeric1;
use nom::branch::alt;
use nom::multi::{many1, many0};
use nom::combinator::cut;
use nom::character::{is_alphanumeric, is_alphabetic};
use nom::sequence::pair;
use wasm_bindgen::prelude::*;

mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Action{
    name:String,
    parameters: Vec<String>
}

fn identifier(text:&str) ->IResult<&str, String>{
    let (text, a) =take_while1(|c| {is_alphabetic(c as u8)||c=='_'})(text)?;
    let (text, b) =take_while(|c| {is_alphanumeric(c as u8)||c=='_'})(text)?;

    Ok((text, format!("{}{}",a,b)))
}

fn parse_action(text:&str) ->IResult<&str, Action>{
    let (text, name) =identifier(text)?;
    let (text, p) =many0(pair(tag("-"),identifier))(text)?;

    Ok((text, Action{name:name, parameters:p.iter().map(|x| x.1.to_string()).collect()}))
}

fn parse4(text:&str) -> IResult<&str, (&str, &str)>{
    let (text, hello) = tag("hello")(text)?;
    let (text, sep) = re_find!(text,"^\\s*,\\s*")?; 
    Ok((text,(hello,sep)))
} 

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, liquer-rust-prototype!");
}

#[wasm_bindgen]
pub fn parse_action1(text:&str) -> JsValue {
    JsValue::from_serde(&parse_action(text).unwrap().1).unwrap()
}
