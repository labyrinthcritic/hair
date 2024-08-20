use crate::primitive::{self, unit};

#[test]
fn identity() {
    let p = primitive::identity();
    assert_eq!(p.parse("hello, world"), Ok(()));
}

#[test]
fn unit_slice() {
    let p = unit();
    let input: &[u32] = &[1, 2, 3];
    assert_eq!(p.parse(input), Ok(&1));
}

#[test]
fn unit_str() {
    let p = unit();
    assert_eq!(p.parse("hello, world"), Ok('h'));
}

#[test]
fn just() {
    assert_eq!(
        primitive::just("hello").parse_at("hello, world!", 0),
        Ok(("hello", 5))
    );
}

#[test]
fn map() {
    let p = unit()
        .filter(char::is_ascii_digit)
        .map(|c| char::to_digit(c, 10).unwrap());

    assert_eq!(p.parse("9876"), Ok(9));
}

#[test]
fn flat_map() {
    let p = unit().flat_map(|c: char| {
        if c == '*' {
            unit().many_with(Some(3), Some(3)).map_err(|_| ()).input()
        } else {
            unit().input()
        }
    });

    assert_eq!(p.parse("-1234"), Ok("1"));
    assert_eq!(p.parse("*1234"), Ok("123"));
}

#[test]
fn filter() {
    let p = unit().filter(|&c| c == 'h');
    assert_eq!(p.parse("hello, world!"), Ok('h'));
    assert_eq!(p.parse("world, hello!"), Err(((), 0)));
}

#[test]
fn filter_map() {
    let digit = unit().filter_map(|c: char| c.to_digit(10));
    assert_eq!(digit.parse("123"), Ok(1));
    assert_eq!(digit.parse("abc"), Err(((), 0)));
}

#[test]
fn sequence() {
    let char = |c: char| unit::<str>().filter(move |&d| c == d);
    let p = char('h').then(char('e').or(char('a')));
    assert_eq!(p.parse("hallo, world!"), Ok(('h', 'a')));
}

#[test]
fn optional() {
    let p = primitive::just("a").optional();

    assert_eq!(p.parse("a"), Ok(Some("a")));
    assert_eq!(p.parse("b"), Ok(None));
}

#[test]
fn many() {
    let char = |c: char| unit::<str>().filter(move |&d| c == d);

    let p = char('a').many();
    assert_eq!(p.parse("aaaaaa"), Ok(vec!['a'; 6]));
}

#[test]
fn separate() {
    let p = primitive::just("0").separate(primitive::just(","));

    assert_eq!(p.parse("0"), Ok(vec!["0"]));
    assert_eq!(p.parse(""), Ok(vec![]));
}

#[test]
fn span_input() {
    let p = unit::<str>().many().input().with_span();
    assert_eq!(p.parse("aaaaa"), Ok(("aaaaa", 0..5)));
}
