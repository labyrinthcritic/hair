# hair

hair is a simple and extensible parser combinator crate that stays out of your way.

## Features

 - Compose parsers in a modular, functional style, and test them independently of one another.

 - Error handling is in the user's control. Parsers have an error type of `()`
   by default; these parsers can be `map_err`-ed to yield more useful errors.
   Compose primitive combinators to create your own 'primitives' that yield
   proper errors.

   ```rust
   use hair::{*, primitive::*};

   struct Token {
     kind: TokenKind,
   }

   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   enum TokenKind {
       Identifier,
   }

   struct Expected(TokenKind);

   use TokenKind::*;
   let kind = |kind: TokenKind| unit::<[Token]>()
       .filter(move |token: &&Token| token.kind == kind)
       .map_err(move |_| Expected(kind));

   let tokens = &[Token { kind: Identifier }];
   let result = kind(Identifier).parse(tokens);
   ````

   (Error propogation is WIP.)

 - No dependencies :)

## Tentative Features

 - While parser combinator crates typically have an API similar to
   Rust's `Iterator` (so, a `Parser` trait with methods that yield
   `Parser`-implementing structs, generic over their dependencies), hair
   provides a [`Parser`] struct wrapper around an owned trait object. This
   sacrifices some potential optimization for simplicity and error messages that
   actually fit on a single screen.

   ```rust
   use hair::{*, primitive::*};
   //         Parser<I,    O,    E> (input, output, error)
   let space: Parser<&str, &str, ()> = just(" ");
   ```

   This paradigm will most likely change in the future. hair's main goal is
   to create as painless an API as possible. (No, *your* trait bounds are not
   satisifed...)

## Etymology

`comb` was already taken.

## Usage

Parsers can operate on any input type - the only constraint is `Clone` (and all
shared references are `Clone`). Primitive combinators like `just` and `unit`
can operate on both slices (`&[T]`) and string slices (`&str`) via the [`Slice`]
trait.

Primitive parsers (the ones you should usually be using as a starting point for
composing parsers) are located in [`primitive`].
