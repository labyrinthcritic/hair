//! Parsers that are not primitives, but may still be useful.

use crate::{primitive::unit, Parser, Slice};

/// Consume one or more units of input while `predicate` is true.
pub fn recognize_input<'a, S, P>(predicate: P) -> Parser<'a, &'a S, &'a S, ()>
where
    S: Slice<'a> + ?Sized,
    P: Fn(&S::Item) -> bool + 'a,
{
    unit()
        .filter(predicate)
        .ignore()
        .many_with(Some(1), None)
        .input()
        .map_err(|_| ())
}
