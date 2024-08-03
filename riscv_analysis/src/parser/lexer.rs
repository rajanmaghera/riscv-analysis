use std::str::Chars;

use uuid::Uuid;

use crate::parser::token::TokenType;
use crate::parser::token::{Position, Range, Token};

use super::{SourceText, WithRangeFromSized};

const EOF_CONST: char = 3 as char;

trait PeekableOverlay: Iterator {
    fn peek(&mut self) -> Option<&<Self as Iterator>::Item>;
    fn next_if(
        &mut self,
        func: impl FnOnce(&<Self as Iterator>::Item) -> bool,
    ) -> Option<<Self as Iterator>::Item> {
        let next = self.peek()?;
        if func(next) {
            Some(self.next()?)
        } else {
            None
        }
    }
}

impl<'a> PeekableOverlay for Chars<'a> {
    fn peek(&mut self) -> Option<&<Self as Iterator>::Item> {
        self.as_str().chars().nth(1).as_ref()
    }
}

/// Lexer for RISC-V assembly
///
/// The lexer implements the Iterator trait, so it can be used in a for loop for
/// getting the next token.
pub struct Lexer<'a> {
    source: Chars<'a>,
    pub source_id: Uuid,
    /// The position that was just read.
    ///
    /// This field will be none if nothing was read. The attached character is the character at that location.
    last_read: Option<(Position, char)>,
}

trait LexerInspection {
    /// Check if the current character is whitespace, excluding newlines.
    ///
    /// This function will return true if the current character is a space,
    /// tab, or comma. Newlines are not considered whitespace as it is a
    /// token in the lexer.
    fn is_lexwhitespace(self) -> bool;

    /// Check if the current character is a character usable in a symbol.
    ///
    /// This function will return true if the current character is a lowercase
    /// or uppercase letter, an underscore, or a dash.
    fn is_symbol_char(self) -> bool;

    /// Check if the current character is a character usable in a symbol
    /// or a digit.
    fn is_symbol_item(self) -> bool;

    fn is_newline(self) -> bool;
}

impl LexerInspection for char {
    fn is_lexwhitespace(self) -> bool {
        self == ' ' || self == '\t' || self == ','
    }

    fn is_symbol_char(self) -> bool {
        self.is_ascii_lowercase() || self.is_ascii_uppercase() || self == '_' || self == '-'
    }

    fn is_symbol_item(self) -> bool {
        self.is_symbol_char() || self.is_ascii_digit()
    }

    fn is_newline(self) -> bool {
        self == '\n'
    }
}

impl<'a> Lexer<'a> {
    /// Create a new lexer from a string.
    pub fn new(source: &str, id: Uuid) -> Lexer {
        Lexer {
            source: source.chars(),
            source_id: id,
            last_read: None,
        }
    }

    pub fn pos(&self) -> Option<Position> {
        Some(self.last_read?.0)
    }

    fn increment_position(&mut self, next_char: char) -> char {
        match self.last_read {
            Some((mut pos, mut char)) => {
                if char == '\n' {
                    pos.add_line();
                } else {
                    pos.add_char();
                }
                char = next_char;
            }
            None => {
                self.last_read = Some((Position::new(0, 0, 0), next_char));
            }
        }
        next_char
    }

    /// Get the next character in the source.
    ///
    /// This function will update the current character and the position
    /// of the Lexer struct.
    #[must_use]
    fn consume_char(&mut self) -> Option<char> {
        Some(self.increment_position(self.source.next()?))
    }

    #[must_use]
    fn consume_char_if(&mut self, func: impl FnOnce(&char) -> bool) -> Option<char> {
        Some(self.increment_position(self.source.next_if(func)?))
    }

    /// Skip all whitespace.
    ///
    /// This will leave the consumable in a spot where the next item will need to be "consumed".
    fn skip_while(&mut self, func: impl Fn(char) -> bool) {
        loop {
            let next_char = match self.source.peek() {
                Some(c) => c,
                None => return,
            };

            if !func(*next_char) {
                break;
            }
        }
    }

    /// Consume while condition is true.
    fn consume_while(&mut self, func: impl Fn(&char) -> bool) -> Option<SourceText> {
        let base_text = self.source.as_str();
        let first_char = self.consume_char_if(func)?;
        let start = self.pos()?;
        while let Some(_) = self.consume_char_if(func) {}
        let end = self.pos()? + 1usize;
        let text = &base_text[..end - start];
        Some(text.with_range_size(start, self.source_id))
    }

    /// Get a range from the current character.
    ///
    /// This function will return a range with the start and end position
    /// being the current position of the lexer.
    fn get_range_of_char(&self) -> Option<Range> {
        Some(Range::new(
            self.pos()?,
            self.pos()? + 1usize,
            self.source_id,
        ))
    }

    fn get_token_of_char(&self, token: TokenType) -> Option<Token> {
        Some(Token::new(token, self.get_range_of_char()?))
    }
}

impl<'a> Lexer<'a> {}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_while(|b| b.is_lexwhitespace());
        loop {
            break match self.consume_char()? {
                '\n' => Some(self.get_token_of_char(TokenType::Newline)?),
                '(' => Some(self.get_token_of_char(TokenType::LParen))?,
                ')' => Some(self.get_token_of_char(TokenType::RParen)?),
                '.' => {
                    let val = self.consume_while(|c| c.is_symbol_item())?;
                    Some(val.into_token(TokenType::Directive(val.text)))
                }
                '#' => {
                    self.skip_while(|b| !b.is_newline());
                    continue;
                }
                '"' => {
                    let val = self.consume_while(|c| *c != '"')?;
                    Some(val.into_token(TokenType::String(val.text)))
                }
                _ => {
                    let val = self.consume_while(|c| c.is_symbol_item())?;

                    if let Some(_) = self.consume_char_if(|c| *c == ':') {
                        Some(val.into_token(TokenType::Label(&val.text)))
                    } else {
                        Some(val.into_token(TokenType::Symbol(&val.text)))
                    }
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::{Lexer, TokenType};
    fn tokenize(input: &str) -> Vec<TokenType> {
        Lexer::new(input, uuid::Uuid::nil())
            .map(|it| it.token)
            .collect()
    }

    #[test]
    fn lex_label() {
        let tokens = tokenize("My_Label:");
        assert_eq!(tokens, vec![TokenType::Label("My_Label")]);
    }

    #[test]
    fn lex_instruction() {
        let tokens = tokenize("add s0, s0, s2");
        assert_eq!(
            tokens,
            vec![
                TokenType::Symbol("add"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s2"),
            ]
        );
    }

    #[test]
    fn lex_ints() {
        let tokens = tokenize("0x1234,    0b1010, 1234  -222");
        assert_eq!(
            tokens,
            vec![
                TokenType::Symbol("0x1234"),
                TokenType::Symbol("0b1010"),
                TokenType::Symbol("1234"),
                TokenType::Symbol("-222"),
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
                TokenType::Symbol("add"),
                TokenType::Symbol("x2"),
                TokenType::Symbol("x2"),
                TokenType::Symbol("x3"),
                TokenType::Newline,
                TokenType::Label("BLCOK"),
                TokenType::Newline,
                TokenType::Newline,
                TokenType::Newline,
                TokenType::Symbol("sub"),
                TokenType::Symbol("a0"),
                TokenType::Symbol("a0"),
                TokenType::Symbol("a1"),
                TokenType::Newline,
                TokenType::Label("my_block"),
                TokenType::Symbol("add"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s2"),
                TokenType::Newline,
                TokenType::Symbol("add"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s2"),
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
                TokenType::Symbol("add"),
                TokenType::Symbol("x2"),
                TokenType::Symbol("x2"),
                TokenType::Symbol("x3"),
                TokenType::Newline,
                TokenType::Label("BLCOK"),
                TokenType::Newline,
                TokenType::Newline,
                TokenType::Newline,
                TokenType::Symbol("sub"),
                TokenType::Symbol("a0"),
                TokenType::Symbol("a0"),
                TokenType::Symbol("a1"),
                TokenType::Newline, // ERROR HERE
                TokenType::Label("my_block"),
                TokenType::Symbol("add"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s2"),
                TokenType::Newline, // ERROR HERE
                TokenType::Symbol("add"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s0"),
                TokenType::Symbol("s2"),
            ]
        );
    }
}
