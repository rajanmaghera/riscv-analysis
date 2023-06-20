use crate::parser::token::Token;
use crate::parser::token::{Info, Position, Range};

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

// While this is not necessary, it is used to skip directives that are
// not text. We should eventually have data directives, but for now
// we will just skip them.

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
        self.ch == ' ' || self.ch == '\t' || self.ch == ','
    }

    fn is_symbol_char(&self) -> bool {
        let c = self.ch;
        // TODO be careful, we may not want - to be a symbol character,
        // This is done so number parsing is only done once we know what the instruction is
        // aka. to make our lives easier
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_' || c == '-'
    }

    fn is_symbol_item(&self) -> bool {
        let c = self.ch;
        self.is_symbol_char() || c.is_ascii_digit()
    }

    fn skip_ws(&mut self) {
        while self.is_space() {
            self.next_char();
        }
    }

    fn get_range(&self) -> Range {
        Range {
            start: Position {
                line: self.row,
                column: self.col,
            },
            end: Position {
                line: self.row,
                column: self.col,
            },
        }
    }

    fn get_pos(&self) -> Position {
        Position {
            line: self.row,
            column: self.col,
        }
    }
}

// TODO typestate for lexer?

impl Iterator for Lexer {
    type Item = Info;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();

        match self.ch {
            '\n' => {
                let pos = self.get_range();
                self.next_char();

                Some(Info {
                    token: Token::Newline,
                    pos,
                })
            }
            '(' => {
                let pos = self.get_range();
                self.next_char();

                Some(Info {
                    token: Token::LParen,
                    pos,
                })
            }
            ')' => {
                let pos = self.get_range();
                self.next_char();

                Some(Info {
                    token: Token::RParen,
                    pos,
                })
            }
            '.' => {
                // directive

                let start = self.get_pos();
                self.next_char();

                let mut dir_str: String = String::new();

                while self.is_symbol_item() {
                    dir_str += &self.ch.to_string();
                    self.next_char();
                }

                let end = self.get_pos();

                if dir_str.is_empty() {
                    // TODO this is an error or end of line?
                    return None;
                }

                Some(Info {
                    token: Token::Directive(dir_str.clone()),
                    pos: Range { start, end },
                })
            }
            '#' => {
                // skip line till newline
                while self.ch != '\n' && self.ch != EOF_CONST {
                    self.next_char();
                }

                if self.ch == EOF_CONST {
                    return None;
                }
                self.next_char();

                // TODO switch to comment tokens
                // For now, we return the newline, as it ends
                // in a newline
                Some(Info {
                    token: Token::Newline,
                    pos: self.get_range(),
                })
            }

            '"' => {
                // string
                let start = self.get_pos();
                let mut string_str: String = String::new();

                self.next_char();

                while self.ch != '"' {
                    string_str += &self.ch.to_string();
                    self.next_char();
                }

                self.next_char();

                let end = self.get_pos();

                self.next_char();

                Some(Info {
                    token: Token::String(string_str.clone()),
                    pos: Range { start, end },
                })
            }
            _ => {
                self.col -= 1;

                let start = self.get_pos();

                let mut symbol_str: String = String::new();

                while self.is_symbol_item() {
                    symbol_str += &self.ch.to_string();
                    self.next_char();
                }

                if symbol_str.is_empty() {
                    // this is an error or end of line?
                    return None;
                } else if self.ch == ':' {
                    // this is a label
                    self.next_char();
                    let end = self.get_pos();

                    return Some(Info {
                        token: Token::Label(symbol_str.clone()),
                        pos: Range { start, end },
                    });
                }

                let end = self.get_pos();

                Some(Info {
                    token: Token::Symbol(symbol_str.clone()),
                    pos: Range { start, end },
                })
            }
        }
    }
}
