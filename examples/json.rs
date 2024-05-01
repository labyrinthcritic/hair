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

#[allow(unused)]
#[derive(Debug)]
enum Expect {
    Char(char),
    String(&'static str),
    Rule(&'static str),
}

fn just<'a>(string: &'static str) -> Parser<'a, &'a str, &'a str, Expect> {
    hair::primitive::just(string).map_err(move |_| Expect::String(string))
}

fn ws<'a>() -> Parser<'a, &'a str, &'a str, Expect> {
    unit()
        .filter(|c: &char| c.is_whitespace())
        .ignore()
        .many()
        .input()
        .map_err(|_| unreachable!())
}

fn string<'a>() -> Parser<'a, &'a str, String, Expect> {
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

fn number<'a>() -> Parser<'a, &'a str, f32, Expect> {
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

// as it is now, recursive parsers must be defined at the function level
fn value(input: &str, at: usize) -> ParseResult<Value, Expect> {
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

fn element<'a>() -> Parser<'a, &'a str, Value, Expect> {
    Parser::new(value).surround(ws(), ws())
}
