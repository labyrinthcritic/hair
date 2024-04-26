//! These can be used as a starting point for building parsers.
//!
//! ## Example
//!
//! ```rust
//! use hair::primitive::{unit, just};
//!
//! let character = |c| unit().filter(move |&d| c == d);
//! assert_eq!(character('a').then(just("bc")).parse("abc"), Some(('a', "bc")));
//! ```

use crate::{Parser, Slice};

/// Successfully parse nothing.
pub fn identity<'a, I: Clone + 'a>() -> Parser<'a, I, ()> {
    Parser::new(|_, at| Some(((), at)))
}

/// Parse and consume a single unit of the input.
/// For `&[T]`, this is `&T`; for `&str`, this is `char`.
pub fn unit<'a, S: Slice<'a> + ?Sized>() -> Parser<'a, &'a S, S::Item> {
    Parser::new(|input: &S, at| {
        let rest = input.index_from(at);
        if let Some((c, len)) = rest.first() {
            Some((c, at + len))
        } else {
            None
        }
    })
}

/// If the remaining input starts with `expected`, output the match.
pub fn just<'a, 'b: 'a, S>(expected: &'b S) -> Parser<'a, &'a S, &'a S>
where
    S: Slice<'a> + PartialEq<S> + ?Sized,
{
    Parser::new(move |input: &S, at| {
        if input.index_from(at).len() >= expected.len()
            && input.index_between(at, at + expected.len()) == expected
        {
            Some((expected, at + expected.len()))
        } else {
            None
        }
    })
}

/// Try all parsers in sequence. Equivalent to `a.or(b).or(c)...`.
pub fn any<'a, I: Clone + 'a, O: 'a, Ps>(parsers: Ps) -> Parser<'a, I, O>
where
    Ps: AsRef<[Parser<'a, I, O>]> + 'a,
{
    Parser::new(move |input: I, at| {
        for parser in parsers.as_ref() {
            if let Some((o, rest)) = parser.parse_at(input.clone(), at) {
                return Some((o, rest));
            }
        }

        None
    })
}
