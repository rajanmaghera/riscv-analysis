use super::{Info, ParserNode};

#[derive(Debug, Clone)]
pub enum ParseError {
    Expected(Vec<ExpectedType>, Info),
    IsNewline(Info),
    Ignored(Info),
    UnexpectedToken(Info),
    UnexpectedEOF,
    NeedTwoNodes(Box<ParserNode>, Box<ParserNode>),
    UnexpectedError,
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
        }
    }
}
