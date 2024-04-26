//! These can be used as a starting point for building parsers.
//!
//! ## Example
//!
//! ```rust
//! use hair::primitive::{unit, just};
//!
//! let character = |c| unit().filter(move |&d| c == d);
//! assert_eq!(character('a').then(just("bc")).parse("abc"), Ok(('a', "bc")));
//! ```

use crate::{Parser, Slice};

/// Successfully parse nothing.
pub fn identity<'a, I: Clone + 'a>() -> Parser<'a, I, (), ()> {
    Parser::new(|_, at| Ok(((), at)))
}

/// Parse and consume a single unit of the input.
/// For `&[T]`, this is `&T`; for `&str`, this is `char`.
pub fn unit<'a, S: Slice<'a> + ?Sized>() -> Parser<'a, &'a S, S::Item, ()> {
    Parser::new(|input: &S, at| {
        let rest = input.index_from(at);
        if let Some((c, len)) = rest.first() {
            Ok((c, at + len))
        } else {
            Err(())
        }
    })
}

/// If the remaining input starts with `expected`, output the match.
pub fn just<'a, 'b: 'a, S>(expected: &'b S) -> Parser<'a, &'a S, &'a S, ()>
where
    S: Slice<'a> + PartialEq<S> + ?Sized,
{
    Parser::new(move |input: &S, at| {
        if input.index_from(at).len() >= expected.len()
            && input.index_between(at, at + expected.len()) == expected
        {
            Ok((expected, at + expected.len()))
        } else {
            Err(())
        }
    })
}

/// Try all parsers in sequence. Equivalent to `a.or(b).or(c)...`.
pub fn any<'a, I: Clone + 'a, O: 'a, E: 'a, Ps>(parsers: Ps) -> Parser<'a, I, O, E>
where
    Ps: AsRef<[Parser<'a, I, O, E>]> + 'a,
{
    Parser::new(move |input: I, at| {
        let mut last_error = None;
        for parser in parsers.as_ref() {
            match parser.parse_at(input.clone(), at) {
                Ok((o, rest)) => return Ok((o, rest)),
                Err(e) => last_error = Some(e),
            }
        }

        Err(last_error.unwrap())
    })
}
