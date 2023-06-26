use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Info {
    pub token: Token,
    pub pos: Range,
}

/// Token type for the parser
///
/// This is the token type for the parser. It is used to
/// determine what the token is, and what to do with it.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// Left Parenthesis '('
    LParen,
    /// Right Parenthesis ')'
    RParen,
    /// Newline '\n'
    Newline,
    /// Label: text ending in ':'
    ///
    /// This is used to mark a label entry point in the code.
    /// It is used to mark the start of a function, or a jump
    /// target.
    Label(String),
    /// Symbol: text not matching any special token types
    ///
    /// This is used to mark a symbol. A symbol is a
    /// generic token that can be converted into a
    /// more specific type. The types include
    /// instructions, registers, numbers, and special CSR numbers/regs.
    Symbol(String),
    /// Directive: text starting with '.'
    ///
    /// This is used to mark a directive. A directive is a
    /// command to the assembler to do something. For example,
    /// the `.text` directive tells the assembler to start
    /// assembling code into the text section.
    ///
    /// The most important directive is `.include`. This
    /// directive tells the assembler to include the file
    /// specified in the directive. This case has to be handled
    /// specially, as the file is not parsed, but rather
    /// included as is.
    Directive(String),
    /// String: text enclosed in double quotes
    String(String),
}

impl PartialEq<Token> for Info {
    fn eq(&self, other: &Token) -> bool {
        self.token == *other
    }
}
impl<T> With<T> {
    pub fn info(&self) -> Info {
        Info {
            token: self.token.clone(),
            pos: self.pos.clone(),
        }
    }
}

#[derive(Clone)]
pub struct With<T> {
    pub token: Token,
    pub pos: Range,
    pub data: T,
}

impl<T> std::fmt::Debug for With<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

impl<T> Hash for With<T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl Display for Info {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Label(s) => writeln!(f, "[label: {s}]"),
            Token::Symbol(s) => write!(f, "<{s}> "),
            Token::Directive(s) => write!(f, "[directive: {s}] "),
            Token::String(s) => write!(f, "\"{s}\""),
            Token::Newline => writeln!(f, "<NL>"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
        }
    }
}

pub struct VecTokenDisplayWrapper<'a>(&'a Vec<Info>);
impl<'a> Display for VecTokenDisplayWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for t in self.0 {
            write!(f, "{t}")?;
        }
        Ok(())
    }
}

pub trait ToDisplayForTokenVec {
    fn to_display(&self) -> VecTokenDisplayWrapper;
}

impl ToDisplayForTokenVec for Vec<Info> {
    fn to_display(&self) -> VecTokenDisplayWrapper {
        VecTokenDisplayWrapper(self)
    }
}

pub trait LineDisplay {
    fn get_range(&self) -> Range;
}

// implement display for Range
impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}:{} - {}:{}",
            self.start.line, self.start.column, self.end.line, self.end.column
        )
    }
}

impl<T> LineDisplay for With<T> {
    fn get_range(&self) -> Range {
        self.pos.clone()
    }
}

impl<T> PartialEq<With<T>> for With<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &With<T>) -> bool {
        self.data == other.data
    }
}

impl<T> Eq for With<T> where T: Eq {}

impl<T> With<T>
where
    T: PartialEq<T>,
{
    pub fn new(data: T, info: Info) -> Self {
        With {
            token: info.token,
            pos: info.pos,
            data,
        }
    }
}

impl<T> TryFrom<Info> for With<T>
where
    T: TryFrom<Info>,
{
    type Error = T::Error;

    fn try_from(value: Info) -> Result<Self, Self::Error> {
        Ok(With {
            pos: value.pos.clone(),
            token: value.token.clone(),
            data: T::try_from(value)?,
        })
    }
}

impl TryFrom<Info> for String {
    type Error = String;

    fn try_from(value: Info) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => Ok(s),
            _ => Err(format!("Expected symbol, got {:?}", value.token)),
        }
    }
}

impl<T> PartialEq<T> for With<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.data == *other
    }
}

trait TokenExpression {
    fn debug_tokens(&self);
}

impl TokenExpression for Vec<Token> {
    fn debug_tokens(&self) {
        print!("Tokens: ");
        for item in self {
            print!("[{item}]");
        }
        println!();
    }
}
