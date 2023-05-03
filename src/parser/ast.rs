use crate::parser::inst::Inst;
use crate::parser::lexer::Lexer;
use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range, Token, TokenInfo, WithToken};
use std::convert::TryFrom;
use std::iter::Peekable;

use super::inst::InstType;

#[derive(Debug, PartialEq, Clone)]
pub struct RType(
    pub WithToken<Register>,
    pub WithToken<Register>,
    pub WithToken<Register>,
);
#[derive(Debug, PartialEq, Clone)]
pub struct IType(
    pub WithToken<Register>,
    pub WithToken<Register>,
    pub WithToken<i32>,
);

#[derive(Debug, PartialEq, Clone)]
pub enum ASTNode {
    Add(RType),
    Sub(RType),
    Addi(IType),
    Subi(IType),
    Branch(String),
    Label(String),
    Ret,
    Call(String),
    Jmp(String),
}

impl ASTNode {
    pub fn is_entry(&self) -> bool {
        match self {
            ASTNode::Label(_) => true,
            _ => false,
        }
    }

    pub fn is_exit(&self) -> bool {
        match self {
            ASTNode::Branch(_) | ASTNode::Ret | ASTNode::Jmp(_) | ASTNode::Call(_) => true,
            _ => false,
        }
    }

    pub fn stores_to(&self) -> Option<Register> {
        match self {
            ASTNode::Add(rtype) => Some(rtype.0.data.clone()),
            ASTNode::Sub(rtype) => Some(rtype.0.data.clone()),
            _ => None,
        }
    }
}

impl LineDisplay for WithToken<ASTNode> {
    fn get_range(&self) -> Range {
        match &self.data {
            ASTNode::Label(s) => self.pos.clone(),
            ASTNode::Add(rtype) => {
                let mut range = self.pos.clone();
                range.end = rtype.2.pos.end;
                range
            }
            _ => unimplemented!(),
        }
    }
}
pub enum ParseError {
    ExpectedRegister,
    UnexpectedToken,
    UnexpectedEOF,
}

impl TryFrom<&mut Peekable<Lexer>> for WithToken<ASTNode> {
    // TODO fix unwraps

    type Error = ParseError;

    fn try_from(value: &mut Peekable<Lexer>) -> Result<Self, Self::Error> {
        use ParseError::*;
        let next_node = value.next().ok_or(UnexpectedEOF)?;
        match &next_node.token {
            Token::Symbol(s) => {
                if let Ok(inst) = Inst::try_from(s) {
                    let node = match InstType::from(&inst) {
                        InstType::RType => {
                            let rd: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let rs1: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let rs2: WithToken<Register> =
                                WithToken::try_from(value.next().ok_or(UnexpectedToken)?).unwrap();
                            let rtype = RType(rd, rs1, rs2);

                            // TODO we should verify this at compile time that the instruction is valid
                            match inst {
                                Inst::Add => Ok(WithToken::new(ASTNode::Add(rtype), next_node)),
                                Inst::Sub => Ok(WithToken::new(ASTNode::Sub(rtype), next_node)),
                                _ => unimplemented!(),
                            }
                        }
                        _ => Err(UnexpectedToken),
                    };
                    return node;
                }
                Err(UnexpectedToken)
            }
            Token::Label(s) => Ok(WithToken::new(ASTNode::Label(s.to_owned()), next_node)),
            _ => Err(UnexpectedToken),
        }
    }
}
