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
    /// The position that will be read next
    pos: usize,
    /// The row that will be read next
    row: usize,
    /// The column that will be read next
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
            match b.get(self.pos) {
                Some(c) => self.ch = *c as char,
                None => self.ch = EOF_CONST,
            }
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
        let mut end = self.get_pos();
        end.column += 1;
        Range {
            start: self.get_pos(),
            end,
        }
    }

    /// Get the current position of the lexer.
    ///
    /// This function will return the current position of the lexer.
    fn get_pos(&self) -> Position {
        let column = if self.col == 0 { 0 } else { self.col - 1 };

        Position {
            line: self.row,
            column,
            raw_index: self.pos,
        }
    }
}

impl Iterator for Lexer {
    type Item = Info;

    #[allow(clippy::too_many_lines)]
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

                    // TODO why isnt this get_pos? has to do with column?
                    let mut end = start;
                    end.column += symbol_str.len();
                    end.raw_index += symbol_str.len();

                    return Some(Info {
                        token: Token::Label(symbol_str.clone()),
                        pos: Range { start, end },
                        file: self.source_id,
                    });
                }

                // TODO why isnt this get_pos? has to do with column?
                let mut end = start;
                end.column += symbol_str.len();
                end.raw_index += symbol_str.len();

                Some(Info {
                    token: Token::Symbol(symbol_str.clone()),
                    pos: Range { start, end },
                    file: self.source_id,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {

    // TODO: These tests only test the token output, but not the ranges or the
    // IDs of the file. Those need to be tested and documented.

    use crate::parser::{Lexer, Token};
    fn tokenize<S: Into<String>>(input: S) -> Vec<Token> {
        Lexer::new(input, uuid::Uuid::nil())
            .map(|x| x.token)
            .collect()
    }

    #[test]
    fn lex_label() {
        let tokens = tokenize("My_Label:");
        assert_eq!(tokens, vec![Token::Label("My_Label".to_owned())]);
    }

    #[test]
    fn lex_directive() {
        let tokens = tokenize(".text");
        assert_eq!(tokens, vec![Token::Directive("text".to_owned())]);
    }

    #[test]
    fn lex_instruction() {
        let tokens = tokenize("add s0, s0, s2");
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("add".to_owned()),
                Token::Symbol("s0".to_owned()),
                Token::Symbol("s0".to_owned()),
                Token::Symbol("s2".to_owned()),
            ]
        );
    }

    #[test]
    fn lex_ints() {
        let tokens = tokenize("0x1234,    0b1010, 1234  -222");
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("0x1234".to_owned()),
                Token::Symbol("0b1010".to_owned()),
                Token::Symbol("1234".to_owned()),
                Token::Symbol("-222".to_owned()),
            ]
        );
    }

    #[test]
    fn lex_long() {
        let tokens = tokenize(
            "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        );
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("add".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x3".into()),
                Token::Newline,
                Token::Label("BLCOK".to_owned()),
                Token::Newline,
                Token::Newline,
                Token::Newline,
                Token::Symbol("sub".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a1".into()),
                Token::Newline,
                Token::Label("my_block".to_owned()),
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
                Token::Newline,
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
            ]
        );
    }

    #[test]
    fn lex_comments() {
        let lexer = tokenize(
            "add x2,x2,x3 # hello, world!@#DKSAOKLJu3iou12o\nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        );

        assert_eq!(
            lexer,
            vec![
                Token::Symbol("add".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x3".into()),
                Token::Newline,
                Token::Label("BLCOK".to_string()),
                Token::Newline,
                Token::Newline,
                Token::Newline,
                Token::Symbol("sub".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a1".into()),
                Token::Newline, // ERROR HERE
                Token::Label("my_block".to_string()),
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
                Token::Newline, // ERROR HERE
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
            ]
        );
    }
}
