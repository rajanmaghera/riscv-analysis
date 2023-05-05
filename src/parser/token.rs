// For tokenizing, we are gonna use the most basic tokens.
// The rest can be done during parsing.
//
// This just makes my job easier. In the future, we may
// want to do this another way.

// TODO use copy over clone

use std::fmt::Display;

use crate::parser::register::Register;

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
pub struct TokenInfo {
    pub token: Token,
    pub pos: Range,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SymbolData(pub String);

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Colon,
    Comma,
    Label(String), // A label has to end with a : without any whitespace
    Symbol(SymbolData),
    Directive(String),
    String(String),
}

impl PartialEq<Token> for TokenInfo {
    fn eq(&self, other: &Token) -> bool {
        self.token == *other
    }
}

#[derive(Debug, Clone)]
pub struct WithToken<T> {
    token: Token,
    pub pos: Range,
    pub data: T,
}

impl Display for TokenInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.token)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Token::Colon => write!(f, "(COLON) "),
            Token::Comma => write!(f, "(COMMA) "),
            Token::Label(s) => write!(f, "[label: {}]\n", s),
            Token::Symbol(s) => write!(f, "<{}> ", s.0),
            Token::Directive(s) => write!(f, "[directive: {}] ", s),
            Token::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

pub struct VecTokenDisplayWrapper<'a>(&'a Vec<TokenInfo>);
impl<'a> Display for VecTokenDisplayWrapper<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for t in self.0 {
            write!(f, "{}", t)?;
        }
        Ok(())
    }
}

pub trait ToDisplayForVecToken {
    fn to_display(&self) -> VecTokenDisplayWrapper;
}

impl ToDisplayForVecToken for Vec<TokenInfo> {
    fn to_display(&self) -> VecTokenDisplayWrapper {
        VecTokenDisplayWrapper(self)
    }
}



pub trait LineDisplay {
    fn get_range(&self) -> Range;
    fn get_str(&self) -> String {
        let s = self.get_range();
        format!(
            "{}:{} - {}:{}",
            s.start.line, s.start.column, s.end.line, s.end.column
        )
    }
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

impl LineDisplay for WithToken<Register> {
    fn get_range(&self) -> Range {
        self.pos.clone()
    }
}

impl<T> PartialEq<WithToken<T>> for WithToken<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &WithToken<T>) -> bool {
        self.data == other.data
    }
}

impl<T> Eq for WithToken<T> where T: Eq {}

impl<T> WithToken<T>
where
    T: PartialEq<T>,
{
    pub fn new(data: T, info: TokenInfo) -> Self {
        WithToken {
            token: info.token,
            pos: info.pos,
            data,
        }
    }

    // TODO should only be used in testing, get rid of later
    pub fn blank(data: T) -> Self {
        WithToken {
            token: Token::Symbol(SymbolData("".to_owned())),
            pos: Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
            data,
        }
    }

    pub fn into(self) -> T {
        self.data
    }
}

impl<T> TryFrom<TokenInfo> for WithToken<T>
where
    T: TryFrom<TokenInfo>,
{
    type Error = T::Error;

    fn try_from(value: TokenInfo) -> Result<Self, Self::Error> {
        Ok(WithToken {
            pos: value.pos.clone(),
            token: value.token.clone(),
            data: T::try_from(value)?,
        })
    }
}

impl<T> PartialEq<T> for WithToken<T>
where
    T: PartialEq<T>,
{
    fn eq(&self, other: &T) -> bool {
        self.data == *other
    }
}

impl From<&str> for SymbolData {
    fn from(s: &str) -> Self {
        SymbolData(s.to_owned())
    }
}

trait TokenExpression {
    fn debug_tokens(&self);
}

impl TokenExpression for Vec<Token> {
    fn debug_tokens(&self) {
        print!("Tokens: ");
        for item in self {
            print!("[{:?}]", item);
        }
        println!("");
    }
}
