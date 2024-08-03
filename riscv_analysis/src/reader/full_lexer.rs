use std::iter::Peekable;

use crate::{parser::Token, reader::Lexer};
use std::iter::Iterator;
use uuid::Uuid;

/// A full RISC-V lexer.
pub struct FullLexer<'a>(Peekable<Lexer<'a>>);
impl<'a> FullLexer<'a> {
    pub fn new(text: &'a str, uuid: Uuid) -> Self {
        FullLexer(Lexer::new(text, uuid).peekable())
    }
}

impl<'a> Iterator for FullLexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> FullLexer<'a> {
    pub fn peek(&mut self) -> Option<&<Self as Iterator>::Item> {
        self.0.peek()
    }
}
