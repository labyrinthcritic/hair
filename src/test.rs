use crate::primitive::{self, unit};

#[test]
fn identity() {
    let p = primitive::identity();
    assert_eq!(p.parse("hello, world"), Some(()));
}

#[test]
fn unit_slice() {
    let p = unit();
    let input: &[u32] = &[1, 2, 3];
    assert_eq!(p.parse(input), Some(&1));
}

#[test]
fn unit_str() {
    let p = unit();
    assert_eq!(p.parse("hello, world"), Some('h'));
}

#[test]
fn just() {
    assert_eq!(
        primitive::just("hello").parse_at("hello, world!", 0),
        Some(("hello", 5))
    );
}

#[test]
fn map() {
    let p = unit()
        .filter(char::is_ascii_digit)
        .map(|c| char::to_digit(c, 10).unwrap());

    assert_eq!(p.parse("9876"), Some(9));
}

#[test]
fn filter() {
    let p = unit().filter(|&c| c == 'h');
    assert_eq!(p.parse("hello, world!"), Some('h'));
    assert_eq!(p.parse("world, hello!"), None);
}

#[test]
fn sequence() {
    let char = |c: char| unit::<str>().filter(move |&d| c == d);
    let p = char('h').then(char('e').or(char('a')));
    assert_eq!(p.parse("hallo, world!"), Some(('h', 'a')));
}

#[test]
fn optional() {
    let p = primitive::just("a").optional();

    assert_eq!(p.parse("a"), Some(Some("a")));
    assert_eq!(p.parse("b"), Some(None));
}

#[test]
fn many() {
    let char = |c: char| unit::<str>().filter(move |&d| c == d);

    let p = char('a').many(..);
    assert_eq!(p.parse("aaaaaa"), Some(vec!['a'; 6]));

    let p = char('a').many(..=4);
    assert_eq!(p.parse("aaaaaa"), Some(vec!['a'; 4]));

    let p = char('a').many(3..);
    assert_eq!(p.parse("aa"), None);
}

#[test]
fn separate() {
    let p = primitive::just("0").separate(primitive::just(","));

    assert_eq!(p.parse("0"), Some(vec!["0"]));
    assert_eq!(p.parse(""), Some(vec![]));
}

#[test]
fn span_input() {
    let p = unit::<str>().many(..=5).input().with_span();
    assert_eq!(p.parse("aaaaaaaaaaaa"), Some(("aaaaa", 0..5)));
}
