use uuid::Uuid;

use crate::parser::token::Token;
use crate::parser::token::{Info, Position, Range};

const EOF_CONST: char = 3 as char;

/// Lexer for RISC-V assembly
///
/// The lexer implements the Iterator trait, so it can be used in a for loop for
/// getting the next token.
pub struct Lexer {
    source: String,
    pub source_id: Uuid,
    ch: char,
    pos: usize,
    row: usize,
    col: usize,
}

impl Lexer {
    /// Create a new lexer from a string.
    pub fn new<S: Into<String>>(source: S, id: Uuid) -> Lexer {
        let mut lex = Lexer {
            source: source.into(),
            source_id: id,
            ch: '\0',
            pos: 0,
            row: 0,
            col: 0,
        };
        lex.next_char();
        lex
    }

    /// Get the next character in the source.
    ///
    /// This function will update the current character and the position
    /// of the Lexer struct.
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

    /// Check if the current character is whitespace, excluding newlines.
    ///
    /// This function will return true if the current character is a space,
    /// tab, or comma. Newlines are not considered whitespace as it is a
    /// token in the lexer.
    fn is_ws(&self) -> bool {
        self.ch == ' ' || self.ch == '\t' || self.ch == ','
    }

    /// Check if the current character is a character usable in a symbol.
    ///
    /// This function will return true if the current character is a lowercase
    /// or uppercase letter, an underscore, or a dash.
    fn is_symbol_char(&self) -> bool {
        let c = self.ch;
        c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_' || c == '-'
    }

    /// Check if the current character is a character usable in a symbol
    /// or a digit.
    fn is_symbol_item(&self) -> bool {
        let c = self.ch;
        self.is_symbol_char() || c.is_ascii_digit()
    }

    /// Skip whitespace.
    ///
    /// This function will skip all whitespace characters, excluding newlines.
    fn skip_ws(&mut self) {
        while self.is_ws() {
            self.next_char();
        }
    }

    /// Get a range from the current character.
    ///
    /// This function will return a range with the start and end position
    /// being the current position of the lexer.
    fn get_range(&self) -> Range {
        Range {
            start: self.get_pos(),
            end: self.get_pos(),
        }
    }

    /// Get the current position of the lexer.
    ///
    /// This function will return the current position of the lexer.
    fn get_pos(&self) -> Position {
        Position {
            line: self.row,
            column: self.col,
        }
    }
}

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
                    file: self.source_id,
                    pos,
                })
            }
            '(' => {
                let pos = self.get_range();
                self.next_char();

                Some(Info {
                    token: Token::LParen,
                    file: self.source_id,
                    pos,
                })
            }
            ')' => {
                let pos = self.get_range();
                self.next_char();

                Some(Info {
                    token: Token::RParen,
                    file: self.source_id,
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
                    return None;
                }

                Some(Info {
                    token: Token::Directive(dir_str.clone()),
                    pos: Range { start, end },
                    file: self.source_id,
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

                Some(Info {
                    token: Token::Newline,
                    pos: self.get_range(),
                    file: self.source_id,
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
                    file: self.source_id,
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
                        file: self.source_id,
                    });
                }

                let end = self.get_pos();

                Some(Info {
                    token: Token::Symbol(symbol_str.clone()),
                    pos: Range { start, end },
                    file: self.source_id,
                })
            }
        }
    }
}
