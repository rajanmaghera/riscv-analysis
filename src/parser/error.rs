use std::fmt::Display;

use uuid::Uuid;

use crate::passes::WarningLevel;

use super::{Info, LineDisplay, ParserNode, With};

#[derive(Debug, Clone)]
/// Lexer error
///
/// These are errors that tell the parser to deal with separately. This do not
/// inherently mean that the code is wrong, but rather that the parser must go
/// down an alternate path to parse the code.
pub enum LexError {
    Expected(Vec<ExpectedType>, Info),
    IsNewline(Info),
    Ignored(Info),
    UnexpectedToken(Info),
    UnexpectedEOF,
    NeedTwoNodes(Box<ParserNode>, Box<ParserNode>),
    UnexpectedError(Info),
    UnknownDirective(Info),
}

#[derive(Debug, Clone)]
/// Parser error
///
/// These are errors that tell the user their code is wrong. These are errors
/// that the parser recovers from by skipping the line, and continuing to parse
/// the rest of the file. The user should see these errors within their editor
pub enum ParseError {
    Expected(Vec<ExpectedType>, Info),
    Unsupported(Info),
    UnexpectedToken(Info),
    UnexpectedError(Info),
    UnknownDirective(Info),
    CyclicDependency(Info),
    FileNotFound(With<String>),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::Expected(expected, _) => {
                write!(
                    f,
                    "Expected one of {}",
                    expected
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ParseError::Unsupported(_) => write!(f, "Unsupported operation"),
            ParseError::UnexpectedToken(_) => write!(f, "Unexpected token"),
            ParseError::UnexpectedError(_) => write!(f, "Unexpected error"),
            ParseError::UnknownDirective(_) => write!(f, "Unknown directive"),
            ParseError::CyclicDependency(_) => write!(f, "Cyclic dependency"),
            ParseError::FileNotFound(file) => write!(f, "File not found: {}", file.data),
        }
    }
}

impl ParseError {
    pub fn file(&self) -> Uuid {
        match self {
            ParseError::Expected(_, info)
            | ParseError::Unsupported(info)
            | ParseError::UnexpectedToken(info)
            | ParseError::UnexpectedError(info)
            | ParseError::UnknownDirective(info)
            | ParseError::CyclicDependency(info) => info.file,
            ParseError::FileNotFound(file) => file.file,
        }
    }
}

impl LineDisplay for ParseError {
    fn get_range(&self) -> super::Range {
        match self {
            ParseError::Expected(_, info)
            | ParseError::Unsupported(info)
            | ParseError::UnexpectedToken(info)
            | ParseError::UnexpectedError(info)
            | ParseError::UnknownDirective(info)
            | ParseError::CyclicDependency(info) => info.pos.clone(),
            ParseError::FileNotFound(file) => file.pos.clone(),
        }
    }
}

impl From<&ParseError> for WarningLevel {
    fn from(e: &ParseError) -> Self {
        match e {
            ParseError::Expected(_, _) => WarningLevel::Error,
            ParseError::Unsupported(_) => WarningLevel::Error,
            ParseError::UnexpectedToken(_) => WarningLevel::Error,
            ParseError::UnexpectedError(_) => WarningLevel::Error,
            ParseError::UnknownDirective(_) => WarningLevel::Error,
            ParseError::CyclicDependency(_) => WarningLevel::Error,
            ParseError::FileNotFound(_) => WarningLevel::Error,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExpectedType {
    Register,
    Imm,
    Label,
    LParen,
    RParen,
    CSRImm,
    Inst,
    String,
}

impl std::fmt::Display for ExpectedType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExpectedType::Register => write!(f, "Register"),
            ExpectedType::Imm => write!(f, "Imm"),
            ExpectedType::Label => write!(f, "Label"),
            ExpectedType::LParen => write!(f, "("),
            ExpectedType::RParen => write!(f, ")"),
            ExpectedType::CSRImm => write!(f, "CSRImm"),
            ExpectedType::Inst => write!(f, "Inst"),
            ExpectedType::String => write!(f, "String"),
        }
    }
}
