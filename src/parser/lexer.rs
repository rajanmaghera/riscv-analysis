use crate::parser::token::{Position, Range, TokenInfo};
use crate::{SymbolData, Token};

const EOF_CONST: char = 3 as char;

pub struct Lexer {
    // TODO use bytes (u8array) for everything here
    // TODO switch to iter methods on bytes
    source: String,
    ch: char,
    pos: usize,
    row: usize,
    col: usize,
}

impl Lexer {
    pub fn new<S: Into<String>>(source: S) -> Lexer {
        let mut lex = Lexer {
            source: source.into(),
            ch: '\0',
            pos: 0,
            row: 0,
            col: 0,
        };
        lex.next_char();
        lex
    }

    pub fn tokenize<S: Into<String>>(input: S) -> Vec<TokenInfo> {
        Lexer::new(input).collect()
    }

    fn next_char(&mut self) {
        let b = self.source.as_bytes();

        if self.pos >= self.source.len() {
            self.ch = EOF_CONST;
        } else {
            self.ch = b[self.pos] as char;
        }

        if self.ch == '\n' {
            self.row += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }

        self.pos += 1;
    }

    fn is_space(&self) -> bool {
        self.ch == ' ' || self.ch == '\t' || self.ch == '\n'
    }

    fn is_symbol_char(&self) -> bool {
        let c = self.ch;
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_symbol_item(&self) -> bool {
        let c = self.ch;
        self.is_symbol_char() || (c >= '0' && c <= '9')
    }

    fn skip_ws(&mut self) {
        while self.is_space() {
            self.next_char();
        }
    }
}

// TODO switch to tokeninfo struct

impl Iterator for Lexer {
    type Item = TokenInfo;

    fn next(&mut self) -> Option<Self::Item> {
                self.skip_ws();
                dbg!(self.ch);
                let token = match self.ch {
                    ':' => Some(TokenInfo {
                        token: Token::Colon,
                        pos: Range {
                            start: Position {
                                line: self.row,
                                column: self.col,
                            },
                            end: Position {
                                line: self.row,
                                column: self.col,
                            },
                        },
                    }),
                    ',' => Some(TokenInfo {
                        token: Token::Comma,
                        pos: Range {
                            start: Position {
                                line: self.row,
                                column: self.col,
                            },
                            end: Position {
                                line: self.row,
                                column: self.col,
                            },
                        },
                    }),
                    '.' => {
                        // directive
                        
                        let start = Position {
                            line: self.row,
                            column: self.col,
                        };
                        self.next_char();

                        let mut dir_str: String = "".to_owned();

                        while self.is_symbol_item() {
                            dir_str += &self.ch.to_string();
                            self.next_char();
                        }

                        dbg!(&dir_str);

                        let end = Position {
                            line: self.row,
                            column: self.col,
                        };

                        if dir_str == "" {
                            // this is an error or end of line?
                            return None;
                        }

                        Some(TokenInfo {
                            token: Token::Directive(dir_str.to_owned()),
                            pos: Range { start, end },
                        })

                    },
                    '#' => {
                        // skip line till newline
                        while self.ch != '\n' {
                            self.next_char();
                        }
                        // TODO eventually we will return a comment token
                        // for now we just skip it
                        // Remember that recursive calls are bad
                        return self.next();
                    },
                },
            }),
            '.' => todo!("directives"),
            _ => {
                let start = Position {
                    line: self.row,
                    column: self.col,
                };

                let mut symbol_str: String = "".to_owned();

                while self.is_symbol_item() {
                    symbol_str += &self.ch.to_string();
                    self.next_char();
                }

                if symbol_str == "" {
                    return None;
                } else if self.ch == ':' {
                    // this is a label
                    self.next_char();
                    let end = Position {
                        line: self.row,
                        column: self.col,
                    };
                    return Some(TokenInfo {
                        token: Token::Label(symbol_str.to_owned()),
                        pos: Range { start, end },
                    });
                }
                Some(TokenInfo {
                    token: Token::Symbol(SymbolData(symbol_str.to_owned())),
                    pos: Range { start, end: start },
                })
            }
        };

        if let Some(_) = token {
            self.next_char();
        }

        token
    }
}
