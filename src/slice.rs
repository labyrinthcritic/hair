//! See [`Slice`].

/// Generalizes types that can be an `I` in some primitive parsers,
/// e.g. [`unit`](crate::primitive::unit).
/// Combinators that operate on `Slice`s will return a
/// `Parser<&S, _> where S: Slice`.
pub trait Slice<'a> {
    type Item;

    fn is_empty(&'a self) -> bool;
    fn len(&'a self) -> usize;
    fn first(&'a self) -> Option<(Self::Item, usize)>;

    /// `slice[..n]`.
    fn index_to(&'a self, n: usize) -> &Self;
    /// `slice[n..]`.
    fn index_from(&'a self, n: usize) -> &Self;
    /// `slice[n..o]`.
    fn index_between(&'a self, n: usize, o: usize) -> &Self;
}

impl<'a, T: 'a> Slice<'a> for [T] {
    type Item = &'a T;

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn first(&'a self) -> Option<(&'a T, usize)> {
        self.first().map(|t| (t, 1))
    }

    fn index_to(&'a self, n: usize) -> &Self {
        &self[..n]
    }

    fn index_from(&'a self, n: usize) -> &Self {
        &self[n..]
    }

    fn index_between(&'a self, n: usize, o: usize) -> &Self {
        &self[n..o]
    }
}

impl<'a> Slice<'a> for str {
    type Item = char;

    fn is_empty(&'a self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn first(&self) -> Option<(char, usize)> {
        self.chars().next().map(|c| (c, c.len_utf8()))
    }

    fn index_to(&'a self, n: usize) -> &Self {
        &self[..n]
    }

    fn index_from(&'a self, n: usize) -> &Self {
        &self[n..]
    }

    fn index_between(&'a self, n: usize, o: usize) -> &Self {
        &self[n..o]
    }
}
