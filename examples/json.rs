use std::collections::HashMap;

use hair::{
    primitive::{any, just, unit},
    ParseResult, Parser,
};

fn main() {
    let json = include_str!("data.json");
    let result = element().parse(json);

    println!("{result:#?}");
}

#[allow(unused)]
#[derive(Debug)]
enum Value {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    String(String),
    Number(f32),
    True,
    False,
    Null,
}

fn ws<'a>() -> Parser<'a, &'a str, &'a str, ()> {
    unit()
        .filter(|c: &char| c.is_whitespace())
        .ignore()
        .many(..)
        .input()
}

fn string<'a>() -> Parser<'a, &'a str, String, ()> {
    just("\\\\")
        .or(just("\\\""))
        .ignore()
        .or(unit::<str>().filter(|&c| c != '"').ignore())
        .many(..)
        .input()
        .surround(just("\""), just("\""))
        .map(String::from)
}

fn number<'a>() -> Parser<'a, &'a str, f32, ()> {
    let digit = unit::<str>().filter(char::is_ascii_digit);

    digit
        .clone()
        .ignore()
        .many(1..)
        .then(just(".").then(digit.ignore().many(1..)).optional())
        .input()
        .map(|n| n.parse().unwrap())
}

// as it is now, recursive parsers must be defined at the function level
fn value(input: &str, at: usize) -> ParseResult<Value, ()> {
    let object = {
        let member = string()
            .surround(ws(), ws())
            .then(just(":").right(element()));

        member
            .separate(just(","))
            .surround(just("{"), just("}"))
            .map(|members| Value::Object(members.into_iter().collect()))
    };

    let array = element()
        .separate(just(","))
        .surround(just("["), just("]"))
        .map(Value::Array);

    any([
        object,
        array,
        just("true").map(|_| Value::True),
        just("false").map(|_| Value::False),
        just("null").map(|_| Value::Null),
        string().map(Value::String),
        number().map(Value::Number),
    ])
    .parse_at(input, at)
}

fn element<'a>() -> Parser<'a, &'a str, Value, ()> {
    Parser::new(value).surround(ws(), ws())
}
