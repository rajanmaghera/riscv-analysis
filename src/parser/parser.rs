use crate::parser::ast::ASTNode;
use crate::parser::lexer::Lexer;
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
    type Item = ASTNode;

    fn next(&mut self) -> Option<Self::Item> {
        // if next item is not a newline and astnode is not a label then error
        ASTNode::try_from(&mut self.lexer).ok()
    }
}
