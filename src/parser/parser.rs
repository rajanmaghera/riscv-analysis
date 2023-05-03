use crate::{ASTNode, Lexer, WithToken};
use std::iter::Peekable;

pub struct Parser {
    lexer: Peekable<Lexer>,
}

impl Parser {
    pub fn new<S: Into<String>>(source: S) -> Parser {
        Parser {
            lexer: Lexer::new(source).peekable(),
        }
    }
}

impl Iterator for Parser {
    type Item = WithToken<ASTNode>;

    fn next(&mut self) -> Option<Self::Item> {
        // if next item is not a newline and astnode is not a label then error
        WithToken::try_from(&mut self.lexer).ok()
    }
}
