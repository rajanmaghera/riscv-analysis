use uuid::Uuid;

use crate::parser::token::Token;
use crate::parser::token::{Info, Position, Range};

use super::LexError;

/// Possible errors when lexing a string.
#[derive(Clone, Debug, PartialEq)]
pub enum StringLexErrorType {
    InvalidEscapeSequence,
    Unclosed,
    Newline,
}

#[derive(Clone, Debug)]
pub struct StringLexError {
    pub pos: Position,
    pub kind: StringLexErrorType,
}

impl StringLexError {
    #[must_use]
    pub fn new(pos: Position, kind: StringLexErrorType) -> Self {
        Self { pos, kind }
    }
}



/// Lexer for RISC-V assembly
///
/// The lexer implements the Iterator trait, so it can be used in a for loop for
/// getting the next token.
pub struct Lexer {
    pub source_id: Uuid,
    /// Raw source, don't read from this directly
    source: Vec<char>,
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
        Lexer {
            source: source.into().chars().collect(),
            source_id: id,
            pos: 0,
            row: 0,
            col: 0,
        }
    }

    /// Get the N'th next character, without updating the current character.
    fn peek(&self, n: usize) -> Option<char> {
        self.source.get(self.pos + n).copied()
    }
    
    /// Get the current next character.
    fn current(&self) -> Option<char> {
        self.peek(0)
    }

    /// Get the next character in the source.
    ///
    /// This function will update the current character and the position
    /// of the Lexer struct.
    fn consume_char(&mut self) {
        // Get the next character
        if let Some(ch) = self.peek(1) {
            // Update the position
            if ch == '\n' {
                self.row += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        } else {
            self.pos = self.source.len();
        }
    }

    /// Skip ahead N characters in the source.
    ///
    /// This function will update the current character and the position of the
    /// lexer.
    fn skip_char(&mut self, n: usize) {
        for _ in 0..n {
            self.consume_char();
        }
    }

    /// Check if the given character is whitespace, excluding newlines.
    ///
    /// This function will return true if the current character is a space,
    /// tab, or comma. Newlines are not considered whitespace as it is a
    /// token in the lexer.
    fn is_ws(ch: char) -> bool {
        ch == ' ' || ch == '\t' || ch == ','
    }

    /// Check if the given character is a character usable in a symbol.
    ///
    /// This function will return true if the current character is a lowercase
    /// or uppercase letter, an underscore, or a dash.
    fn is_symbol_char(ch: char) -> bool {
        ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch == '_' || ch == '-'
    }

    /// Check if the given character is a character usable in a symbol
    /// or a digit.
    fn is_symbol_item(ch: char) -> bool {
        Self::is_symbol_char(ch) || ch.is_ascii_digit()
    }

    /// Skip whitespace.
    ///
    /// This function will skip all whitespace characters, excluding newlines.
    fn skip_ws(&mut self) {
        while let Some(current) = self.current() {
            if !Self::is_ws(current) {
                break;
            }
            self.consume_char();
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

    /// Lex a unicode escape code.
    ///
    /// Returns None if the code doesn't define a valid unicode character. The
    /// escape code is lexed into a single unicode character.
    fn unicode_code(&mut self) -> Option<char> {
        let chars = vec![
            self.peek(2)?,
            self.peek(3)?,
            self.peek(4)?,
            self.peek(5)?,
        ];

        // Convert to a codepoint number
        let code_number: Option<u32> = {
            let mut acc = 0;
            for c in chars {
                acc *= 16;
                match c.to_digit(16) {
                    Some(d) => acc += d,
                    None => return None,
                }
            }
            Some(acc)
        };

        let c = char::from_u32(code_number?)?;
        self.skip_char(4);
        Some(c)
    }

    /// Lex an escape code.
    ///
    /// All escape codes present in RARS are supported. This function will
    /// consume all needed characters for the escape code.
    fn escape_code(&mut self) -> Option<char> {
        if let Some(c) = self.peek(1) {
            let real = match c {
                '\\' => '\\',
                '\'' => '\'',
                '"'  => '"',
                'n'  => '\n',
                't'  => '\t',
                'r'  => '\r',
                'b'  => '\x08',  // Backspace
                'f'  => '\x0c',  // Form feed
                '0'  => '\0',
                'u'  => self.unicode_code()?,
                // TODO: Unicode input
                _ => return None,
            };

            self.consume_char();
            return Some(real)
        }

        None
    }

    /// Accumulate a string.
    ///
    /// This function handles the string escape codes available in RARS. Due to
    /// escape codes, the number of characters in the string may be less than
    /// the source range.
    fn acc_string(&mut self) -> Result<String, StringLexError> {
        let mut acc: String = String::new();


        while let Some(current) = self.current() {
            if current == '"' {
                return Ok(acc);
            }

            // All strings must be on a single line
            if current == '\n' {
                return Err(
                    StringLexError::new(self.get_pos(), StringLexErrorType::Newline)
                );
            }

            // Check if this is an escape sequence
            if current == '\\' {
                match self.escape_code() {
                    Some(ec) => acc.push(ec),
                    None => return Err(
                        StringLexError::new(self.get_pos(), StringLexErrorType::InvalidEscapeSequence)
                    ),
                }
            }

            // Otherwise, add the character
            else {
                acc.push(current);
            }
            self.consume_char();
        }

        // If we run out of characters, we have an un-closed string
        Err(StringLexError::new(self.get_pos(), StringLexErrorType::Unclosed))
    }

    /// Create the error for an invalid string.
    fn invalid_string(&self, partial: String, kind: StringLexErrorType, start: Position, end: Position) -> Result<Info, LexError> {
        Err(LexError::InvalidString(
            Info {
                token: Token::String(partial),
                pos: Range { start, end },
                file: self.source_id,
            },
            Box::new(StringLexError::new(end, kind))
        ))
    }
}

impl Iterator for Lexer {
    type Item = Result<Info, LexError>;

    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();

        // TODO(rajan): ensure that we are consistent with whether the tokens are included or not in the Token representation
        // TODO(rajan): should we introduce a new token type for the comment hash (#) and directive hash (.)?

        let token = match self.current() {
            None => None,
            Some('\n') => {
                let pos = self.get_range();

                self.consume_char();

                Some(Info {
                    token: Token::Newline,
                    file: self.source_id,
                    pos,
                })
            }
            Some('(') => {
                let pos = self.get_range();
                self.consume_char();

                Some(Info {
                    token: Token::LParen,
                    file: self.source_id,
                    pos,
                })
            }
            Some(')') => {
                let pos = self.get_range();
                self.consume_char();

                Some(Info {
                    token: Token::RParen,
                    file: self.source_id,
                    pos,
                })
            }
            Some('.') => {
                // directive
                let start = self.get_pos();
                let mut dir_str: String = String::new();

                while let Some(current) = self.current() {
                    dir_str.push(current);
                    if let Some(next) = self.peek(1) {
                        if !Self::is_symbol_char(next) {
                            break;
                        }
                    }
                    self.consume_char();
                }

                let end = self.get_pos();
                self.consume_char();

                if dir_str == "." {
                    return self.next();
                }

                Some(Info {
                    token: Token::Directive(dir_str.clone()),
                    pos: Range { start, end },
                    file: self.source_id,
                })
            }
            Some('#') => {
                // Convert comments to token
                let start = self.get_pos();
                let mut comment_str: String = String::new();

                while let Some(current) = self.current() {
                    comment_str.push(current);
                    if self.peek(1) == Some('\n') || self.peek(1).is_none() {
                        break;
                    }
                    self.consume_char();
                }

                let end = self.get_pos();
                self.consume_char();

                // Remove the '#' character
                let (_, comment_str) = comment_str.split_at(1);

                // Empty comment strings are allowed, in the case of a
                // comment with a new line. We don't strip any whitespace
                // for comments here.
                Some(Info {
                    token: Token::Comment(comment_str.to_string()),
                    pos: Range { start, end },
                    file: self.source_id,
                })
            }
            Some('"') => {
                // string
                let start = self.get_pos();
                self.consume_char();   // Skip the first quote

                let string_str = match self.acc_string() {
                    Ok(s) => s,
                    Err(e) => {
                        return Some(Err(LexError::InvalidString(
                            Info {
                                token: Token::String(String::new()),
                                pos: Range { start, end: e.pos },
                                file: self.source_id,
                            },
                            Box::new(e)
                        )));
                    },
                };

                let end = self.get_pos();
                self.consume_char();   // Skip final '"'
                self.consume_char();

                Some(Info {
                    token: Token::String(string_str.clone()),
                    pos: Range { start, end },
                    file: self.source_id,
                })
            }
            Some('\'') => {
                let start = self.get_pos();
                self.consume_char();

                if let Some(c) = self.current() {
                    // Get the character in the quote
                    let c = match c {
                        // Is an escape code
                        '\\' => match self.escape_code() {
                            Some(ec) => ec,
                            None => return Some(self.invalid_string(
                                c.to_string(),
                                StringLexErrorType::InvalidEscapeSequence,
                                start, self.get_pos())
                            )
                        },
                        // Can't have a literal newline in a character
                        '\n' => return Some(self.invalid_string(
                            c.to_string(),
                            StringLexErrorType::Newline,
                            start,
                            self.get_pos()
                        )),
                        // Otherwise, return the character as is
                        c => c,
                    };

                    // Ensure that the next character is the closing quote
                    self.consume_char();
                    if let Some(eq) = self.current() {

                        // Return the character
                        if eq == '\'' {
                            let end = self.get_pos();
                            self.consume_char();

                            return Some(Ok(Info {
                                token: Token::Char(c),
                                pos: Range { start, end },
                                file: self.source_id,
                            }));
                        }

                        // The character is unclosed
                        let end = self.get_pos();
                        return Some(self.invalid_string(
                            c.to_string(),
                            StringLexErrorType::Unclosed,
                            start, end
                        ));
                    }
                }

                let end = self.get_pos();
                return Some(self.invalid_string(
                    String::new(), // Empty string, since we are at EOF
                    StringLexErrorType::Unclosed,
                    start, end
                ));
            }
            _ => {
                // symbol
                let start = self.get_pos();
                let mut symbol_str: String = String::new();

                // If the first character is not a symbol char -> error
                if let Some(current) = self.current() {
                    if !Self::is_symbol_item(current) {
                        return None;
                    }
                }

                while let Some(current) = self.current() {
                    symbol_str.push(current);
                    if let Some(next) = self.peek(1) {
                        if !Self::is_symbol_item(next) {
                            break;
                        }
                    }
                    self.consume_char();
                }

                // If the next char is ':', this is a label
                if self.peek(1) == Some(':') {
                    self.consume_char();   // Move onto the ':'
                    let end = self.get_pos();
                    self.consume_char();

                    return Some(Ok(Info {
                        token: Token::Label(symbol_str.clone()),
                        pos: Range { start, end },
                        file: self.source_id,
                    }));
                }

                let end = self.get_pos();
                self.consume_char();

                Some(Info {
                    token: Token::Symbol(symbol_str.clone()),
                    pos: Range { start, end },
                    file: self.source_id,
                })
            }
        };

        match token {
            Some(t) => {
                // TODO: remove these debug asserts once we fix the get_pos() function
                debug_assert_eq!(t.pos.start.line, t.pos.end.line);
                debug_assert!(t.pos.start.column <= t.pos.end.column);
                Some(Ok(t))
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {

    // TODO: These tests only test the token output, but not the ranges or the
    // IDs of the file. Those need to be tested and documented.

    use crate::parser::{Info, LexError, Lexer, StringLexErrorType, Token};
    fn tokenize<S: Into<String>>(input: S) -> Vec<Token> {
        Lexer::new(input, uuid::Uuid::nil())
            .map(|x| x.unwrap().token) // All tokens should be valid
            .collect()
    }

    fn tokenize_err<S: Into<String>>(input: S) -> Vec<Result<Info, LexError>> {
        Lexer::new(input, uuid::Uuid::nil())
            .collect()
    }

    #[test]
    fn lex_label() {
        let tokens = tokenize("My_Label:");
        assert_eq!(tokens, vec![Token::Label("My_Label".to_owned())]);
    }

    #[test]
    fn lex_comment() {
        let tokens = tokenize("# comments are needed");
        assert_eq!(
            tokens,
            vec![Token::Comment(" comments are needed".to_owned())]
        );
    }

    #[test]
    fn lex_comments_with_differing_whitespaces() {
        let tokens =
            tokenize("#\n#\n# new line comments  with lots of \t whitespace and other special .text characters is allowed  jal ra, x0   \n\n  #.text\n#li a0, 0");
        assert_eq!(
            tokens,
            vec![
                Token::Comment("".to_owned()),
                Token::Newline,
                Token::Comment("".to_owned()),
                Token::Newline,
                Token::Comment(" new line comments  with lots of \t whitespace and other special .text characters is allowed  jal ra, x0   ".to_owned()),
                Token::Newline,
                Token::Newline,
                Token::Comment(".text".to_owned()),
                Token::Newline,
                Token::Comment("li a0, 0".to_owned()),
            ]
        );
    }

    #[test]
    fn lex_empty_comment_as_final_character() {
        let tokens = tokenize("#this is a comment\n#");
        assert_eq!(
            tokens,
            vec![
                Token::Comment("this is a comment".to_owned()),
                Token::Newline,
                Token::Comment("".to_owned()),
            ]
        )
    }

    #[test]
    fn lex_directive() {
        let tokens = tokenize(".text");
        assert_eq!(tokens, vec![Token::Directive(".text".to_owned())]);
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
    fn lex_all_tokens() {
        let lexer = tokenize(
            ".text add x2,x2,x3 # hello, world!@#DKSAOKLJu3iou12o\nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        );

        assert_eq!(
            lexer,
            vec![
                Token::Directive(".text".to_string()),
                Token::Symbol("add".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x3".into()),
                Token::Comment(" hello, world!@#DKSAOKLJu3iou12o".to_string()),
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

    #[test]
    fn lex_all_tokens_with_newlines() {
        let lexer = tokenize(
            ".text\nadd x2,x2,x3 \n# hello, world!@#DKSAOKLJu3iou12o\nBLCOK:\n\n\nsub a0 a0 a1\nmy_block:\nadd s0, s0, s2\nadd s0, s0, s2\nlabel_abc: \n",
        );

        assert_eq!(
            lexer,
            vec![
                Token::Directive(".text".to_string()),
                Token::Newline,
                Token::Symbol("add".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x3".into()),
                Token::Newline,
                Token::Comment(" hello, world!@#DKSAOKLJu3iou12o".to_string()),
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
                Token::Newline,
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
                Token::Newline, // ERROR HERE
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
                Token::Newline,
                Token::Label("label_abc".to_string()),
                Token::Newline,
            ]
        );
    }

    #[test]
    fn strings() {
        let input = r#""" "abcde" "\\\'\"\n\t\r\b\f\0\u03bb" "#;
        let tokens = tokenize(input);

        assert_eq!(
            tokens,
            vec![
                Token::String("".into()),
                Token::String("abcde".into()),
                Token::String("\\'\"\n\t\r\u{8}\u{c}\0\u{03bb}".into()),
            ]
        );
    }

    #[test]
    fn unbounded_string() {
        let input = "\"Good string\" \"Bad string";
        let tokens = tokenize_err(input);

        assert_eq!(tokens.len(), 2);

        // First token should be "Good string"
        assert!(matches!(
            &tokens[0],
            Ok(info) if matches!(
                &info.token,
                Token::String(s) if s == "Good string"
            )
        ));

        // Second token should have failed
        assert!(matches!(
            &tokens[1],
            Err(err) if matches!(
                err,
                LexError::InvalidString(_info, sub)
                    if sub.kind == StringLexErrorType::Unclosed
            )
        ));
    }

    #[test]
    fn newline_in_string() {
        let input = "\"Before \n\"";
        let tokens = tokenize_err(input);

        // After the newline, the lexer will attempt to lex the rest of the
        // input. Thus we should see the following tokens:
        // - An error for the partial string before the newline.
        // - A newline token.
        // - A unclosed string, since the input at this point is a double quote.
        assert_eq!(tokens.len(), 3);

        assert!(matches!(
            &tokens[0],
            Err(err) if matches!(
                err,
                LexError::InvalidString(_info, sub)
                    if sub.kind == StringLexErrorType::Newline
            )
        ));

        assert!(matches!(
            &tokens[1],
            Ok(info) if matches!(&info.token, Token::Newline)
        ));

        assert!(matches!(
            &tokens[2],
            Err(err) if matches!(
                err,
                LexError::InvalidString(_info, sub)
                    if sub.kind == StringLexErrorType::Unclosed
            )
        ));
    }

    #[test]
    fn invalid_escape_code() {
        let input = "\"\\a\"";
        let tokens = tokenize_err(input);

        assert_eq!(tokens.len(), 1);

        assert!(matches!(
            &tokens[0],
            Err(err) if matches!(
                err,
                LexError::InvalidString(_info, sub)
                    if sub.kind == StringLexErrorType::InvalidEscapeSequence
            )
        ));
    }
}
