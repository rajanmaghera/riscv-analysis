use crate::parser::ast::ASTNode;
use crate::parser::lexer::Lexer;
use std::iter::Peekable;

use super::ast::ParseError;
use super::token::Token;

pub struct Parser {
    lexer: Peekable<Lexer>,
}

impl Parser {
    pub fn new<S: Into<String>>(source: S) -> Parser {
        Parser {
            lexer: Lexer::new(source).peekable(),
        }
    }

    // if there is an error, we will try to recover from it
    // by skipping the rest of the line
    fn recover_from_parse_error(&mut self) {
        while let Some(token) = self.lexer.next() {
            if token == Token::Newline {
                break;
            }
        }
    }
}

impl Iterator for Parser {
    type Item = ASTNode;

    fn next(&mut self) -> Option<Self::Item> {
        // if next item is not a newline and astnode is not a label then error
        loop {
            let mut item = ASTNode::try_from(&mut self.lexer);

            // if item is an ast parse error, then keep trying
            while let Err(ParseError::IsNewline) = item {
                item = ASTNode::try_from(&mut self.lexer);
            }

            return match item {
                Ok(ast) => Some(ast),
                Err(err) => match err {
                    ParseError::UnexpectedEOF => None,
                    ParseError::IsNewline => todo!(),
                    _ => {
                        self.recover_from_parse_error();
                        continue;
                    }
                },
            };
        }
    }
}
