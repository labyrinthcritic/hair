//! A JSON parser that yields somewhat reasonable errors. Its entry point is [`element`].
//! JSON's grammar is defined at <https://json.org>.
//! Note that this parser does not consider hex numbers, exponents, or signs.

use std::collections::HashMap;

use hair::{
    primitive::{any, unit},
    ParseResult, Parser,
};

fn main() {
    let json = include_str!("data.json");
    let result = element().parse(json);

    println!("{result:#?}");
}

/// A JSON value.
#[allow(unused)]
#[derive(Debug)]
pub enum Value {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
    String(String),
    Number(f32),
    True,
    False,
    Null,
}

/// The error yielded by this parser.
#[allow(unused)]
#[derive(Debug)]
pub enum Expect {
    Char(char),
    String(&'static str),
    Rule(&'static str),
}

fn just<'a>(string: &'static str) -> Parser<'a, &'a str, &'a str, Expect> {
    hair::primitive::just(string).map_err(move |_| Expect::String(string))
}

pub fn ws<'a>() -> Parser<'a, &'a str, &'a str, Expect> {
    unit()
        .filter(|c: &char| c.is_whitespace())
        .ignore()
        .many()
        .input()
        .map_err(|_| unreachable!())
}

pub fn string<'a>() -> Parser<'a, &'a str, String, Expect> {
    let u = just("\\\\")
        .or(just("\\\""))
        .ignore()
        .ignore_err()
        .or(unit::<str>().filter(|&c| c != '"').ignore());

    u.many()
        .input()
        .map_err(|_| Expect::Rule("string"))
        .surround(just("\""), just("\"").expect())
        .map(String::from)
}

pub fn number<'a>() -> Parser<'a, &'a str, f32, Expect> {
    let digit = || unit::<str>().filter(char::is_ascii_digit);
    let digits = || {
        digit()
            .then(digit().ignore().many())
            .input()
            .map_err(|_| Expect::Rule("digit"))
    };

    digits()
        .then(just(".").then(digits()).optional())
        .input()
        .map(|n| n.parse().unwrap())
}

pub fn value<'a>() -> Parser<'a, &'a str, Value, Expect> {
    // recursive parsers can be defined with an inner function
    fn inner(input: &str, at: usize) -> ParseResult<Value, Expect> {
        let object = {
            let member = string()
                .surround(ws(), ws())
                .then(just(":").expect().right(element().expect()));

            member
                .separate(just(","))
                .surround(just("{"), just("}").expect())
                .map(|members| Value::Object(members.into_iter().collect()))
        };

        let array = element()
            .separate(just(","))
            .surround(just("["), just("]").expect())
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

    Parser::new(inner)
}

pub fn element<'a>() -> Parser<'a, &'a str, Value, Expect> {
    value().surround(ws(), ws())
}
