#![doc = include_str!("../README.md")]

pub mod primitive;
pub mod slice;

#[cfg(test)]
mod test;

use std::{
    ops::{Bound, Range, RangeBounds},
    rc::Rc,
};

pub use slice::Slice;

pub type ParseResult<O, E> = Result<(O, usize), E>;

/// Trait object of a parsing function.
pub type ParseFn<'a, I, O, E> = dyn Fn(I, usize) -> ParseResult<O, E> + 'a;

/// The type of any parser.
/// This is a newtype around [`ParseFn`].
/// To run the parser, call [`Parser::parse`].
#[must_use = "parsers are lazy; call `Parser::parse` to use them"]
pub struct Parser<'a, I, O, E> {
    run: Rc<ParseFn<'a, I, O, E>>,
}

impl<'a, I, O, E> Clone for Parser<'a, I, O, E> {
    fn clone(&self) -> Self {
        Self {
            run: Rc::clone(&self.run),
        }
    }
}

impl<'a, I: Clone + 'a, O: 'a, E: 'a> Parser<'a, I, O, E> {
    pub fn new<P>(p: P) -> Self
    where
        P: Fn(I, usize) -> ParseResult<O, E> + 'a,
    {
        Parser { run: Rc::new(p) }
    }

    /// Parse starting at an offset. This should be used when calling a parser
    /// inside another parser.
    pub fn parse_at(&self, i: I, n: usize) -> ParseResult<O, E> {
        (self.run)(i, n)
    }

    /// Parse from the beginning, and collect the output.
    pub fn parse(&self, i: I) -> Result<O, E> {
        self.parse_at(i, 0).map(|(o, _)| o)
    }

    /// Map the parser's output, i.e. turn a `Parser<I, O, E>` to a `Parser<I, O1, E>`.
    pub fn map<O1: 'a, F>(self, f: F) -> Parser<'a, I, O1, E>
    where
        F: Fn(O) -> O1 + 'a,
    {
        Parser::new(move |input, at| self.parse_at(input, at).map(|(o, rest)| (f(o), rest)))
    }

    /// Map the parser's error, if any, i.e. turn a `Parser<I, O, E>` to a `Parser<I, O, E1>`.
    pub fn map_err<E1: 'a, F>(self, f: F) -> Parser<'a, I, O, E1>
    where
        F: Fn(E) -> E1 + 'a,
    {
        Parser::new(move |input, at| self.parse_at(input, at).map_err(&f))
    }

    /// Make a parser fail if its output does not satisfy `predicate`.
    pub fn filter<P>(self, predicate: P) -> Parser<'a, I, O, ()>
    where
        P: Fn(&O) -> bool + 'a,
    {
        Parser::new(move |input, at| match self.parse_at(input, at) {
            Ok((o, rest)) if predicate(&o) => Ok((o, rest)),
            Ok((_, _)) | Err(_) => Err(()),
        })
    }

    /// Parse with `self`; on failure, parse with `other`.
    pub fn or<E1: 'a>(self, other: Parser<'a, I, O, E1>) -> Parser<'a, I, O, E1> {
        Parser::new(move |input: I, at| {
            self.parse_at(input.clone(), at)
                .or_else(|_| other.parse_at(input, at))
        })
    }

    /// Parse with `self`, then parse the remaining input with `other`,
    /// gathering both outputs into a tuple.
    pub fn then<O1: 'a>(self, snd: Parser<'a, I, O1, E>) -> Parser<'a, I, (O, O1), E> {
        Parser::new(move |input: I, at| {
            let (o, rest) = self.parse_at(input.clone(), at)?;
            let (o1, rest) = snd.parse_at(input, rest)?;
            Ok(((o, o1), rest))
        })
    }

    /// Parse with `self`, then parse with `right`, ignoring its output and
    /// returning the output of self.
    pub fn left<O1: 'a>(self, right: Parser<'a, I, O1, E>) -> Parser<'a, I, O, E> {
        Parser::new(move |input: I, at| {
            let (o, rest) = self.parse_at(input.clone(), at)?;
            let (_, rest) = right.parse_at(input, rest)?;
            Ok((o, rest))
        })
    }

    /// Parse with `self`, ignoring its output, then parse with `right`,
    /// returning its output.
    pub fn right<O1: 'a>(self, right: Parser<'a, I, O1, E>) -> Parser<'a, I, O1, E> {
        Parser::new(move |input: I, at| {
            let (_, rest) = self.parse_at(input.clone(), at)?;
            let (o, rest) = right.parse_at(input, rest)?;
            Ok((o, rest))
        })
    }

    /// Make this parser optional. This parser will always succeed.
    pub fn optional(self) -> Parser<'a, I, Option<O>, E> {
        Parser::new(move |input, at| {
            if let Ok((o, rest)) = self.parse_at(input, at) {
                Ok((Some(o), rest))
            } else {
                Ok((None, at))
            }
        })
    }

    /// Surround a parser with delimiter parsers.
    pub fn surround<OLeft: 'a, ORight: 'a>(
        self,
        left: Parser<'a, I, OLeft, E>,
        right: Parser<'a, I, ORight, E>,
    ) -> Parser<'a, I, O, E> {
        left.right(self).left(right)
    }

    /// Repeat this parser indefinitely, until either the parser fails,
    /// or the upper bound of `range` is reached.
    pub fn many<R: RangeBounds<usize> + 'a>(self, range: R) -> Parser<'a, I, Vec<O>, ()> {
        Parser::new(move |input: I, mut at| {
            let mut os = Vec::new();
            while let Ok((o, rest)) = self.parse_at(input.clone(), at) {
                os.push(o);
                at = rest;

                if match range.end_bound() {
                    Bound::Included(&b) => os.len() > b - 1,
                    Bound::Excluded(&b) => os.len() >= b - 1,
                    Bound::Unbounded => false,
                } {
                    break;
                }
            }

            if range.contains(&os.len()) {
                Ok((os, at))
            } else {
                Err(())
            }
        })
    }

    /// Parse zero or more `self`s, separated with `by`. This allows a trailing
    /// separator.
    pub fn separate<O1: 'a>(self, by: Parser<'a, I, O1, E>) -> Parser<'a, I, Vec<O>, ()> {
        // i'm unsatisfied with this implementation
        // TODO: allow for ranges like `many`, and make trailing separator configurable
        Parser::new(move |input: I, mut at| {
            let mut os = Vec::new();
            loop {
                if let Ok((o, rest)) = self.parse_at(input.clone(), at) {
                    os.push(o);
                    at = rest;
                } else {
                    break;
                }

                if let Ok((_, rest)) = by.parse_at(input.clone(), at) {
                    at = rest;
                } else {
                    break;
                }
            }

            Ok((os, at))
        })
    }

    /// Drop this parser's output.
    pub fn ignore(self) -> Parser<'a, I, (), E> {
        self.map(|_| ())
    }

    /// Associate the output with the range of indices the parser consumed.
    pub fn with_span(self) -> Parser<'a, I, (O, Range<usize>), E> {
        Parser::new(move |input, at| {
            let (o, rest) = self.parse_at(input, at)?;
            Ok(((o, at..rest), rest))
        })
    }
}

/// Implementations on parsers that accept slices as input.
impl<'a, S: Slice<'a> + ?Sized, O: 'a, E: 'a> Parser<'a, &'a S, O, E> {
    pub fn input(self) -> Parser<'a, &'a S, &'a S, E> {
        Parser::new(move |input, at| {
            let (_, rest) = self.parse_at(input, at)?;
            Ok((input.index_between(at, rest), rest))
        })
    }
}
