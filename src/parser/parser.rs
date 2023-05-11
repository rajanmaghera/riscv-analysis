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

// TODO errors are alright, but they do not account for multiple paths
// ie. when we use an if let Ok( ) =, we ignore the error the first time, but
// we do not ignore it the second time. I want both errors to be caught and
// reported.

impl Iterator for Parser {
    type Item = ASTNode;

    fn next(&mut self) -> Option<Self::Item> {
        // if next item is not a newline and astnode is not a label then error
        loop {
            let mut item = ASTNode::try_from(&mut self.lexer);

            // if item is an ast parse error, then keep trying
            while let Err(ParseError::IsNewline(_)) = item {
                item = ASTNode::try_from(&mut self.lexer);
            }

            // print debug info for errors
            match &item {
                Err(err) => match err {
                    ParseError::UnexpectedEOF => {}
                    ParseError::IsNewline(_) => {}
                    ParseError::ExpectedImm(x) => {
                        println!("line {}: Expected immediate value", x.pos.start.line)
                    }
                    ParseError::ExpectedRegister(x) => {
                        println!("line {}: Expected register", x.pos.start.line)
                    }
                    ParseError::ExpectedLabel(x) => {
                        println!("line {}: Expected label", x.pos.start.line)
                    }
                    ParseError::ExpectedMem(x) => {
                        println!("line {}: Expected memory address", x.pos.start.line)
                    }
                    ParseError::UnexpectedToken(x) => {
                        println!("line {}: Unexpected token {:?}", x.pos.start.line, x.token)
                    }
                    _ => {}
                },
                _ => {}
            }

            return match item {
                Ok(ast) => Some(ast),
                Err(err) => match err {
                    ParseError::UnexpectedEOF => None,
                    ParseError::IsNewline(_) => {
                        self.recover_from_parse_error();
                        continue;
                    }
                    _ => {
                        self.recover_from_parse_error();
                        continue;
                    }
                },
            };
        }
    }
}
