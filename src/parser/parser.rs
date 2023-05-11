use crate::parser::ast::ASTNode;
use crate::parser::lexer::Lexer;
use std::collections::VecDeque;
use std::iter::Peekable;

use super::ast::ParseError;
use super::token::Token;

pub struct Parser {
    lexer: Peekable<Lexer>,
    queue: VecDeque<ASTNode>,
}

impl Parser {
    pub fn new<S: Into<String>>(source: S) -> Parser {
        Parser {
            lexer: Lexer::new(source).peekable(),
            queue: VecDeque::new(),
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
        // if there is an item in the queue, return it
        if let Some(item) = self.queue.pop_front() {
            return Some(item);
        }

        loop {
            let mut item = ASTNode::try_from(&mut self.lexer);

            // if item is an ast parse error, then keep trying
            while let Err(ParseError::IsNewline(_)) = item {
                item = ASTNode::try_from(&mut self.lexer);
            }

            // print debug info for errors
            match &item {
                Err(err) => match err {
                    ParseError::Expected(tokens, found) => {
                        println!("Expected {:?}, found {:?}", tokens, found)
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
                    ParseError::NeedTwoNodes(node1, node2) => {
                        self.queue.push_back(node2);
                        Some(node1)
                    }
                    ParseError::UnexpectedEOF => None,
                    _ => {
                        self.recover_from_parse_error();
                        continue;
                    }
                },
            };
        }
    }
}
