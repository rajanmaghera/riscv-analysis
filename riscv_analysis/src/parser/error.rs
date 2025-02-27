use std::fmt::Display;

use uuid::Uuid;

use crate::{
    passes::{DiagnosticLocation, DiagnosticMessage, SeverityLevel},
    reader::FileReaderError,
};

use super::{Token, ParserNode, StringLexError, StringLexErrorType, With};

#[derive(Debug, Clone)]
/// Lexer error
///
/// These are errors that tell the parser to deal with separately. This do not
/// inherently mean that the code is wrong, but rather that the parser must go
/// down an alternate path to parse the code.
pub enum LexError {
    Expected(Vec<ExpectedType>, Token),
    IsNewline(Token),
    IgnoredWithWarning(Token),
    IgnoredWithoutWarning,
    UnexpectedToken(Token),
    UnexpectedEOF,
    NeedTwoNodes(Box<ParserNode>, Box<ParserNode>),
    UnexpectedError(Token),
    UnknownDirective(Token),
    UnsupportedDirective(Token),
    InvalidString(Token, Box<StringLexError>),
}

#[derive(Debug, Clone)]
/// Parser error
///
/// These are errors that tell the user their code is wrong. These are errors
/// that the parser recovers from by skipping the line, and continuing to parse
/// the rest of the file. The user should see these errors within their editor
pub enum ParseError {
    Expected(Vec<ExpectedType>, Token),
    Unsupported(Token),
    UnexpectedToken(Token),
    UnexpectedError(Token),
    UnknownDirective(Token),
    CyclicDependency(Token),
    FileNotFound(With<String>),
    IOError(With<String>, String),
    InvalidString(Token, Box<StringLexError>),
}

impl FileReaderError {
    #[must_use]
    pub fn to_parse_error(&self, path: With<String>) -> ParseError {
        match self {
            FileReaderError::InternalFileNotFound | FileReaderError::Unexpected => {
                ParseError::UnexpectedError(path.info())
            }
            FileReaderError::FileAlreadyRead(_) => ParseError::CyclicDependency(path.info()),
            FileReaderError::InvalidPath => ParseError::FileNotFound(path),
            FileReaderError::IOErr(e) => ParseError::IOError(path, e.clone()),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::Expected(expected, found) => {
                write!(
                    f,
                    "Expected {}, found {}",
                    expected
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" or "),
                    found.token
                )
            }
            ParseError::Unsupported(_) => write!(f, "Unsupported operation"),
            ParseError::UnexpectedToken(_) => write!(f, "Unexpected token"),
            ParseError::UnexpectedError(_) => write!(f, "Unexpected error"),
            ParseError::UnknownDirective(_) => write!(f, "Unknown directive"),
            ParseError::CyclicDependency(_) => write!(f, "Cyclic dependency"),
            ParseError::FileNotFound(file) => write!(f, "File not found: {}", file.data),
            ParseError::IOError(file, err) => write!(f, "IO Error: {} ({})", file.data, err),
            ParseError::InvalidString(_info, _kind) => {
                write!(f, "Invalid string")
            }
        }
    }
}

impl DiagnosticMessage for ParseError {
    fn related(&self) -> Option<Vec<crate::passes::RelatedDiagnosticItem>> {
        None
    }
    fn level(&self) -> SeverityLevel {
        self.into()
    }
    fn title(&self) -> String {
        self.to_string()
    }
    fn description(&self) -> String {
        self.long_description()
    }
    fn long_description(&self) -> String {
        match self {
            ParseError::Expected(expected, found) => format!(
                "Expected {0}, found {1}.\n\n\
                The program found a {1} when it expected a {0}. This might be due to a typo or\
                the wrong or unsupported itembeing entered.",
                expected
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" or "),
                found.to_string()
            ),
            ParseError::Unsupported(_) => "Unsupported operation.\n\n\
                This token or directive is not supported by this program. Please file a bug report or ignore\
                this error.".to_string(),
            ParseError::UnexpectedToken(_) => "Unexpected token.\n\n\
            This token was not expected here. This is likely a typo or an unsupported item."
                .to_string(),
            ParseError::UnexpectedError(_) => {
                "Unexpected error. Please file a bug preport.".to_string()
            }
            ParseError::UnknownDirective(token) => format!("Unknown directive {0}\n\n\
                This directive is not recognized by the program. Please file a bug report or ignore this error.
            ", token.token),
            ParseError::CyclicDependency(_) => "There is a cyclic dependency between files.\n\n\
                This is likely due to a file importing itself or a file importing a file that imports it.\
                Please remove the cyclic dependency to fix this error.
            ".to_string(),
            ParseError::FileNotFound(file) => format!("File not found: {}", file.data),
            ParseError::IOError(file, err) => format!("IO Error: {} ({})", file.data, err),
            ParseError::InvalidString(_, e) => {
                match e.kind {
                    StringLexErrorType::InvalidEscapeSequence => {
                        "String contains an invalid escape sequence".to_string()
                    }
                    StringLexErrorType::Unclosed => {
                        "String is missing closing quote".to_string()
                    }
                    StringLexErrorType::Newline => {
                        "String spans multiple lines".to_string()
                    }
                }
            }
        }
    }
}

impl PartialEq for dyn DiagnosticLocation {
    fn eq(&self, other: &dyn DiagnosticLocation) -> bool {
        self.range() == other.range() && self.file() == other.file()
    }
}

impl PartialOrd for dyn DiagnosticLocation {
    fn partial_cmp(&self, other: &dyn DiagnosticLocation) -> Option<std::cmp::Ordering> {
        if self.file() == other.file() {
            self.range().partial_cmp(&other.range())
        } else {
            None
        }
    }
}

impl DiagnosticLocation for ParseError {
    fn file(&self) -> Uuid {
        match self {
            ParseError::Expected(_, info)
            | ParseError::Unsupported(info)
            | ParseError::UnexpectedToken(info)
            | ParseError::UnexpectedError(info)
            | ParseError::UnknownDirective(info)
            | ParseError::InvalidString(info, _)
            | ParseError::CyclicDependency(info) => info.file,
            ParseError::FileNotFound(file) | ParseError::IOError(file, _) => file.file,
        }
    }

    fn range(&self) -> super::Range {
        match self {
            ParseError::Expected(_, info)
            | ParseError::Unsupported(info)
            | ParseError::UnexpectedToken(info)
            | ParseError::UnexpectedError(info)
            | ParseError::UnknownDirective(info)
            | ParseError::InvalidString(info, _)
            | ParseError::CyclicDependency(info) => info.pos.clone(),
            ParseError::FileNotFound(file) | ParseError::IOError(file, _) => file.pos.clone(),
        }
    }
}

impl From<&ParseError> for SeverityLevel {
    fn from(e: &ParseError) -> Self {
        match e {
            ParseError::Expected(_, _)
            | ParseError::Unsupported(_)
            | ParseError::UnexpectedToken(_)
            | ParseError::UnexpectedError(_)
            | ParseError::UnknownDirective(_)
            | ParseError::CyclicDependency(_)
            | ParseError::FileNotFound(_)
            | ParseError::InvalidString(..)
            | ParseError::IOError(_, _) => SeverityLevel::Error,
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
            ExpectedType::Register => write!(f, "REGISTER"),
            ExpectedType::Imm => write!(f, "IMMEDIATE"),
            ExpectedType::Label => write!(f, "LABEL"),
            ExpectedType::LParen => write!(f, "LPAREN"),
            ExpectedType::RParen => write!(f, "RPAREN"),
            ExpectedType::CSRImm => write!(f, "CSR-IMMEDIATE"),
            ExpectedType::Inst => write!(f, "INSTRUCTION"),
            ExpectedType::String => write!(f, "STRING"),
        }
    }
}
