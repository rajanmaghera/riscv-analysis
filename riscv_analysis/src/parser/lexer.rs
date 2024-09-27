use uuid::Uuid;

use crate::parser::token::Token;
use crate::parser::token::{Info, Position, Range};

const EOF_CONST: char = '\x03';

/// Lexer for RISC-V assembly
///
/// The lexer implements the Iterator trait, so it can be used in a for loop for
/// getting the next token.
pub struct Lexer {
    pub source_id: Uuid,
    /// Raw source, don't read from this directly
    source: Vec<char>,
    /// Current character
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
            source: source.into().chars().collect(),
            source_id: id,
            ch: '\0',
            pos: 0,
            row: 0,
            col: 0,
        };
        lex.ch = lex.peek(0);
        lex
    }

    /// Get the N'th next character, without updating the current character.
    fn peek(&self, n: usize) -> char {
        match self.source.get(self.pos + n) {
            Some(c) => *c,
            None => EOF_CONST,
        }
    }
    
    /// Get the current next character.
    fn current(&self) -> char {
        self.ch
    }

    /// Get the next character in the source.
    ///
    /// This function will update the current character and the position
    /// of the Lexer struct.
    fn next_char(&mut self) {
        // Get the next character
        self.ch = self.peek(1);

        // Update the position
        if self.ch == '\n' {
            self.row += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }

        self.pos += 1;
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
        while Self::is_ws(self.current()) {
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

    /// Accumulate a string.
    ///
    /// This function handles the string escape codes available in RARS.
    fn acc_string(&mut self) -> String {
        let mut acc: String = String::new();

        // If this char is a quote, we have the empty string
        if self.current() == '"' {
            return acc;
        }

        loop {
            // Check if this is an escape sequence
            if self.current() == '\\' {
                let c = match self.peek(1) {
                    '\\' =>'\\',
                    '\'' =>'\'',
                    '"'  =>'"',
                    'n'  =>'\n',
                    't'  =>'\t',
                    'r'  =>'\r',
                    'b'  =>'\x08',  // Backspace
                    'f'  =>'\x0c',  // Form feed
                    '0'  =>'\0',
                    _ => self.current(),
                };
                acc.push(c);
                self.next_char(); // Skip the code
            }

            // Otherwise, add the character
            else {
                acc.push(self.current());
            }

            if self.peek(1) == '"' {
                break;
            }
            self.next_char();
        }
        acc
    }
}

impl Iterator for Lexer {
    type Item = Info;

    #[allow(clippy::too_many_lines)]
    fn next(&mut self) -> Option<Self::Item> {
        self.skip_ws();

        // TODO(rajan): ensure that we are consistent with whether the tokens are included or not in the Token representation
        // TODO(rajan): should we introduce a new token type for the comment hash (#) and directive hash (.)?

        let token = match self.current() {
            EOF_CONST => None,
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
                let mut dir_str: String = String::new();

                loop {
                    dir_str.push(self.current());
                    if !Self::is_symbol_char(self.peek(1)) {
                        break;
                    }
                    self.next_char();
                }

                let end = self.get_pos();
                self.next_char();

                if dir_str == "." {
                    return self.next();
                }

                Some(Info {
                    token: Token::Directive(dir_str.clone()),
                    pos: Range { start, end },
                    file: self.source_id,
                })
            }
            '#' => {
                // Convert comments to token
                let start = self.get_pos();
                let mut comment_str: String = String::new();

                loop {
                    comment_str.push(self.current());
                    if self.peek(1) == '\n' || self.peek(1) == EOF_CONST {
                        break;
                    }
                    self.next_char();
                }

                let end = self.get_pos();
                self.next_char();

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

            '"' => {
                // string
                let start = self.get_pos();
                self.next_char();   // Skip the first quote

                let string_str = self.acc_string();

                let end = self.get_pos();
                self.next_char();   // Skip final '"'
                self.next_char();

                Some(Info {
                    token: Token::String(string_str.clone()),
                    pos: Range { start, end },
                    file: self.source_id,
                })
            }
            _ => {
                // symbol
                let start = self.get_pos();
                let mut symbol_str: String = String::new();

                // If the first character is not a symbol char -> error
                if !Self::is_symbol_item(self.current()) {
                    return None;
                }

                loop {
                    symbol_str.push(self.current());
                    if !Self::is_symbol_item(self.peek(1)) {
                        break;
                    }
                    self.next_char();
                }

                // If the next char is ':', this is a label
                if self.peek(1) == ':' {
                    self.next_char();   // Move onto the ':'
                    let end = self.get_pos();
                    self.next_char();

                    return Some(Info {
                        token: Token::Label(symbol_str.clone()),
                        pos: Range { start, end },
                        file: self.source_id,
                    });
                }

                let end = self.get_pos();
                self.next_char();

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
                Some(t)
            }
            None => None,
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
}
