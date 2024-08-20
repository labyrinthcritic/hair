#![doc = include_str!("../README.md")]

pub mod primitive;
pub mod slice;
pub mod util;

#[cfg(test)]
mod test;

use std::{ops::Range, rc::Rc};

pub use slice::Slice;

pub type ParseResult<O, E> = Result<(O, usize), Error<E>>;

/// Trait object of a parsing function.
pub type ParseFn<'a, I, O, E> = dyn Fn(I, usize) -> ParseResult<O, E> + 'a;

/// The type of any parser, a wrapper for a [`ParseFn`] object.
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
    pub fn parse(&self, i: I) -> Result<O, (E, usize)> {
        self.parse_at(i, 0)
            .map(|(o, _)| o)
            .map_err(|Error { inner, at, .. }| (inner, at))
    }

    /// Map the parser's output, i.e. turn a `Parser<I, O, E>` into a `Parser<I, O1, E>`.
    pub fn map<O1: 'a, F>(self, f: F) -> Parser<'a, I, O1, E>
    where
        F: Fn(O) -> O1 + 'a,
    {
        Parser::new(move |input, at| self.parse_at(input, at).map(|(o, rest)| (f(o), rest)))
    }

    /// Map the parser's error, if any, i.e. turn a `Parser<I, O, E>` into a `Parser<I, O, E1>`.
    pub fn map_err<E1: 'a, F>(self, f: F) -> Parser<'a, I, O, E1>
    where
        F: Fn(E) -> E1 + 'a,
    {
        Parser::new(move |input, at| {
            self.parse_at(input, at)
                .map_err(|Error { inner, recover, at }| Error {
                    inner: f(inner),
                    recover,
                    at,
                })
        })
    }

    pub fn flat_map<O1: 'a, F>(self, f: F) -> Parser<'a, I, O1, E>
    where
        F: Fn(O) -> Parser<'a, I, O1, E> + 'a,
    {
        Parser::new(move |input: I, at| {
            let (o, at) = self.parse_at(input.clone(), at)?;
            f(o).parse_at(input, at)
        })
    }

    /// Make a parser yield a fatal error on failure. This should be used in
    /// situations where a previous parser guarantees that this is the only
    /// correct parsing path.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // since key-value pairs only exist in objects, `:` will always come after
    /// // the identifier, and a value will always come after the `:`.
    /// let key_value_pair = identifier()
    ///     .then(just(":").expect())
    ///     .then(value().expect());
    ///
    /// let object = just("{").then(key_value_pair.separate(just(","))).then("}");
    /// ```
    ///
    /// Where `identifier` and `value` are user-defined parsers.
    pub fn expect(self) -> Parser<'a, I, O, E> {
        Parser::new(move |input, at| match self.parse_at(input, at) {
            o @ Ok(_) => o,
            Err(err) => Err(err.fail()),
        })
    }

    /// Make a parser fail if its output does not satisfy `predicate`.
    pub fn filter<P>(self, predicate: P) -> Parser<'a, I, O, ()>
    where
        P: Fn(&O) -> bool + 'a,
    {
        Parser::new(move |input, at| match self.parse_at(input, at) {
            Ok((o, rest)) if predicate(&o) => Ok((o, rest)),
            Ok(_) | Err(_) => Err(Error {
                inner: (),
                recover: Recover::Recoverable,
                at,
            }),
        })
    }

    /// Parse with `self`; on failure, parse with `other`.
    /// Fatal errors will short-circuit.
    pub fn or(self, other: Parser<'a, I, O, E>) -> Parser<'a, I, O, E> {
        Parser::new(move |input: I, at| match self.parse_at(input.clone(), at) {
            Ok(ok) => Ok(ok),
            Err(err) => match err.recover {
                Recover::Recoverable => other.parse_at(input, at),
                Recover::Fatal => Err(err),
            },
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

    /// Make this parser optional. Succeeds on recoverable errors.
    pub fn optional(self) -> Parser<'a, I, Option<O>, E> {
        Parser::new(move |input, at| match self.parse_at(input, at) {
            Ok((o, rest)) => Ok((Some(o), rest)),
            Err(err) => match err.recover {
                Recover::Recoverable => Ok((None, at)),
                Recover::Fatal => Err(err),
            },
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

    /// Repeat this parser indefinitely until failure.
    /// This is equivalent to `.many_with(None, None)`.
    pub fn many(self) -> Parser<'a, I, Vec<O>, E> {
        self.many_with(None, None).map_err(|e| e.unwrap())
    }

    /// Repeat this parser until `at_most` is met. If the parser fails before
    /// `at_least` outputs were collected, the parser will return Err(None).
    pub fn many_with(
        self,
        at_least: Option<usize>,
        at_most: Option<usize>,
    ) -> Parser<'a, I, Vec<O>, Option<E>> {
        Parser::new(move |input: I, at| {
            let mut os = Vec::new();
            let mut rest = at;
            loop {
                if at_most.is_some_and(|max| os.len() >= max) {
                    break;
                }

                match self.parse_at(input.clone(), rest) {
                    Ok((o, r)) => {
                        os.push(o);
                        rest = r;
                    }
                    Err(err) => match err.recover {
                        Recover::Recoverable => break,
                        Recover::Fatal => return Err(err.map(Some)),
                    },
                }
            }

            if at_least.is_some_and(|min| os.len() < min) {
                Err(Error {
                    inner: None,
                    recover: Recover::Recoverable,
                    // TODO: at was mutated, is this correct?
                    at,
                })
            } else {
                Ok((os, rest))
            }
        })
    }

    /// Parse zero or more `self`s, separated with `by`. This allows a trailing
    /// separator.
    pub fn separate<O1: 'a>(self, by: Parser<'a, I, O1, E>) -> Parser<'a, I, Vec<O>, E> {
        Parser::new(move |input: I, mut at| {
            let mut os = Vec::new();
            loop {
                match self.parse_at(input.clone(), at) {
                    Ok((o, rest)) => {
                        os.push(o);
                        at = rest;
                    }
                    Err(err) => match err.recover {
                        Recover::Recoverable => break,
                        Recover::Fatal => return Err(err.fail()),
                    },
                }

                match by.parse_at(input.clone(), at) {
                    Ok((_, rest)) => {
                        at = rest;
                    }
                    Err(err) => match err.recover {
                        Recover::Recoverable => break,
                        Recover::Fatal => return Err(err.fail()),
                    },
                }
            }

            Ok((os, at))
        })
    }

    /// Drop this parser's output.
    pub fn ignore(self) -> Parser<'a, I, (), E> {
        self.map(|_| ())
    }

    /// Drop this parser's error.
    pub fn ignore_err(self) -> Parser<'a, I, O, ()> {
        self.map_err(|_| ())
    }

    /// Associate the output with the range of indices that the parser consumed.
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

/// This type wraps errors as they propagate upward through parsers. `E` is the
/// parser's actual error type, whether it be `()` or a user-defined error.
///
/// hair borrows the error propagation mechanism seen in some other combinator
/// libraries, such as nom. Errors have a state of 'recoverable' or 'fatal',
/// where fatal errors will always propagate upward regardless of alternatives.
/// hair's primitive combinators will never yield a fatal error - it is
/// up to the user to decide which parsers should throw fatal errors with
/// [`Parser::expect`].
///
/// Introducing fatal-throwing parsers will never cause another failing parser
/// to succeed. It is only for providing more accurate error messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error<E> {
    inner: E,
    recover: Recover,
    at: usize,
}

impl<E> Error<E> {
    pub fn new(inner: E, at: usize) -> Self {
        Self {
            inner,
            recover: Recover::Recoverable,
            at,
        }
    }

    /// Map the error's inner value.
    pub fn map<F, E1>(self, f: F) -> Error<E1>
    where
        F: Fn(E) -> E1,
    {
        let Error { inner, recover, at } = self;
        Error {
            inner: f(inner),
            recover,
            at,
        }
    }

    /// Make this error fatal.
    #[must_use]
    pub fn fail(self) -> Error<E> {
        Error {
            recover: Recover::Fatal,
            ..self
        }
    }
}

/// State within [`Error`]. Errors with `Recover::Fatal` short-circuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Recover {
    Recoverable,
    Fatal,
}
