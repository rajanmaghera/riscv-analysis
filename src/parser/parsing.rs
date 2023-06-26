use crate::parser::inst::{
    ArithType, BranchType, CSRIType, CSRType, IArithType, Inst, JumpLinkRType, JumpLinkType,
    PseudoType, Type,
};
use crate::parser::token::With;
use crate::parser::ParseError;
use crate::parser::ParserNode;
use crate::parser::Register;
use crate::parser::{DirectiveType, Lexer};
use std::iter::Peekable;
use std::{collections::VecDeque, str::FromStr};

use super::imm::{CSRImm, Imm};
use super::token::{Info, Token};
use super::{ExpectedType, LabelString};

/// Parser for RISC-V assembly
pub struct Parser {
    lexer: Peekable<Lexer>,
    queue: VecDeque<ParserNode>,
}

impl Parser {
    pub fn new<S: Into<String>>(source: S) -> Parser {
        Parser {
            lexer: Lexer::new(source).peekable(),
            queue: VecDeque::new(),
        }
    }

    /// Skip the rest of the line
    ///
    /// This is used to recover from parse errors. If there is a parse error,
    /// we will skip the rest of the line and try to parse the next line.
    fn recover_from_parse_error(&mut self) {
        for token in self.lexer.by_ref() {
            if token == Token::Newline {
                break;
            }
        }
    }
}

impl Iterator for Parser {
    type Item = ParserNode;

    fn next(&mut self) -> Option<Self::Item> {
        // if there is an item in the queue, return it
        if let Some(item) = self.queue.pop_front() {
            return Some(item);
        }

        loop {
            let mut item = ParserNode::try_from(&mut self.lexer);

            // if item is an ast parse error, then keep trying
            while let Err(ParseError::IsNewline(_)) = item {
                item = ParserNode::try_from(&mut self.lexer);
            }

            // print debug info for errors
            // if let Err(err) = &item {
            //     match err {
            //         ParseError::Expected(tokens, found) => {
            //             println!(
            //                 "Expected {}, found {found}",
            //                 tokens
            //                     .iter()
            //                     .map(std::string::ToString::to_string)
            //                     .collect::<Vec<String>>()
            //                     .join(" or "),
            //             );
            //         }
            //         ParseError::UnexpectedToken(x) => {
            //             println!("line {}: Unexpected token {}", x.pos.start.line, x.token);
            //         }
            //         _ => {}
            //     }
            // }
            return match item {
                Ok(ast) => Some(ast),
                Err(err) => match err {
                    ParseError::NeedTwoNodes(node1, node2) => {
                        self.queue.push_back(*node2);
                        Some(*node1)
                    }
                    ParseError::UnexpectedEOF => None,
                    _ => {
                        self.recover_from_parse_error();
                        continue;
                    }
                },
            };
        }
    }
}

fn expect_lparen(value: Option<Info>) -> Result<(), ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    match v.token {
        Token::LParen => Ok(()),
        _ => Err(ParseError::Expected(vec![ExpectedType::LParen], v)),
    }
}

fn expect_rparen(value: Option<Info>) -> Result<(), ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    match v.token {
        Token::RParen => Ok(()),
        _ => Err(ParseError::Expected(vec![ExpectedType::RParen], v)),
    }
}

fn get_reg(value: Option<Info>) -> Result<With<Register>, ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    With::<Register>::try_from(v.clone())
        .map_err(|_| ParseError::Expected(vec![ExpectedType::Register], v))
}

fn get_imm(value: Option<Info>) -> Result<With<Imm>, ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    With::<Imm>::try_from(v.clone()).map_err(|_| ParseError::Expected(vec![ExpectedType::Imm], v))
}

fn get_label(value: Option<Info>) -> Result<With<LabelString>, ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    With::<LabelString>::try_from(v.clone())
        .map_err(|_| ParseError::Expected(vec![ExpectedType::Label], v))
}

fn get_csrimm(value: Option<Info>) -> Result<With<CSRImm>, ParseError> {
    let v = value.ok_or(ParseError::UnexpectedEOF)?;
    With::<CSRImm>::try_from(v.clone())
        .map_err(|_| ParseError::Expected(vec![ExpectedType::CSRImm], v))
}

impl TryFrom<&mut Peekable<Lexer>> for ParserNode {
    // TODO ensure that symbol is not a register
    // TODO how is error handling handled for non registers

    type Error = ParseError;

    #[allow(clippy::too_many_lines)]
    fn try_from(value: &mut Peekable<Lexer>) -> Result<Self, Self::Error> {
        use ParseError::{Expected, Ignored, IsNewline, NeedTwoNodes, UnexpectedEOF};
        let next_node = value.next().ok_or(UnexpectedEOF)?;
        match &next_node.token {
            Token::Symbol(s) => {
                if let Ok(inst) = Inst::from_str(s) {
                    let node = match Type::from(&inst) {
                        Type::CsrI(inst) => {
                            let rd = get_reg(value.next())?;
                            let csr = get_csrimm(value.next())?;
                            let imm = get_imm(value.next())?;
                            Ok(ParserNode::new_csri(
                                With::new(inst, next_node),
                                rd,
                                csr,
                                imm,
                            ))
                        }
                        Type::Csr(inst) => {
                            let rd = get_reg(value.next())?;
                            let csr = get_csrimm(value.next())?;
                            let rs1 = get_reg(value.next())?;
                            Ok(ParserNode::new_csr(
                                With::new(inst, next_node),
                                rd,
                                csr,
                                rs1,
                            ))
                        }
                        Type::UpperArith(inst) => {
                            let rd = get_reg(value.next())?;
                            let imm = get_imm(value.next())?;
                            Ok(ParserNode::new_upper_arith(
                                With::new(inst, next_node),
                                rd,
                                imm,
                            ))
                        }
                        Type::Arith(inst) => {
                            let rd = get_reg(value.next())?;
                            let rs1 = get_reg(value.next())?;
                            let rs2 = get_reg(value.next())?;
                            Ok(ParserNode::new_arith(
                                With::new(inst, next_node),
                                rd,
                                rs1,
                                rs2,
                            ))
                        }
                        Type::IArith(inst) => {
                            let rd = get_reg(value.next())?;
                            let rs1 = get_reg(value.next())?;
                            let imm = get_imm(value.next())?;
                            Ok(ParserNode::new_iarith(
                                With::new(inst, next_node),
                                rd,
                                rs1,
                                imm,
                            ))
                        }

                        Type::JumpLink(inst) => {
                            let name_token = value.next();

                            return if let Ok(reg) = get_reg(name_token.clone()) {
                                let name = get_label(value.next())?;
                                Ok(ParserNode::new_jump_link(
                                    With::new(inst, next_node),
                                    reg,
                                    name,
                                ))
                            } else if let Ok(name) = get_label(name_token.clone()) {
                                Ok(ParserNode::new_jump_link(
                                    With::new(inst, next_node.clone()),
                                    With::new(Register::X1, next_node),
                                    name,
                                ))
                            } else {
                                Err(Expected(
                                    vec![ExpectedType::Register, ExpectedType::Label],
                                    name_token.ok_or(UnexpectedEOF)?,
                                ))
                            };
                        }
                        Type::JumpLinkR(inst) => {
                            let reg1 = get_reg(value.next())?;
                            let next = value.next();
                            return if let Ok(rs1) = get_reg(next.clone()) {
                                let imm = get_imm(value.next())?;
                                Ok(ParserNode::new_jump_link_r(
                                    With::new(inst, next_node),
                                    reg1,
                                    rs1,
                                    imm,
                                ))
                            } else if let Ok(imm) = get_imm(next.clone()) {
                                if let Ok(()) = expect_lparen(value.peek().cloned()) {
                                    value.next();
                                    let rs1 = get_reg(value.next())?;
                                    expect_rparen(value.next())?;
                                    Ok(ParserNode::new_jump_link_r(
                                        With::new(inst, next_node),
                                        reg1,
                                        rs1,
                                        imm,
                                    ))
                                } else {
                                    Ok(ParserNode::new_jump_link_r(
                                        With::new(inst, next_node.clone()),
                                        With::new(Register::X1, next_node),
                                        reg1,
                                        imm,
                                    ))
                                }
                            } else if let Ok(()) = expect_lparen(next) {
                                let rs1 = get_reg(value.next())?;
                                expect_rparen(value.next())?;
                                Ok(ParserNode::new_jump_link_r(
                                    With::new(inst, next_node.clone()),
                                    reg1,
                                    rs1,
                                    With::new(Imm(0), next_node),
                                ))
                            } else {
                                Ok(ParserNode::new_jump_link_r(
                                    With::new(inst, next_node.clone()),
                                    With::new(Register::X1, next_node.clone()),
                                    reg1,
                                    With::new(Imm(0), next_node),
                                ))
                            };
                        }
                        Type::Load(inst) => {
                            let rd = get_reg(value.next())?;
                            let next = value.next();
                            return if let Ok(imm) = get_imm(next.clone()) {
                                if let Ok(()) = expect_lparen(value.peek().cloned()) {
                                    value.next();
                                    let rs1 = get_reg(value.next())?;
                                    expect_rparen(value.next())?;
                                    Ok(ParserNode::new_load(
                                        With::new(inst, next_node),
                                        rd,
                                        rs1,
                                        imm,
                                    ))
                                } else {
                                    Ok(ParserNode::new_load(
                                        With::new(inst, next_node.clone()),
                                        rd,
                                        With::new(Register::X0, next_node),
                                        imm,
                                    ))
                                }
                            } else if let Ok(label) = get_label(next.clone()) {
                                Err(NeedTwoNodes(
                                    Box::new(ParserNode::new_load_addr(
                                        With::new(PseudoType::La, next_node.clone()),
                                        rd.clone(),
                                        label,
                                    )),
                                    Box::new(ParserNode::new_load(
                                        With::new(inst, next_node.clone()),
                                        rd.clone(),
                                        rd,
                                        With::new(Imm(0), next_node),
                                    )),
                                ))
                            } else if let Ok(()) = expect_lparen(next.clone()) {
                                let rs1 = get_reg(value.next())?;
                                expect_rparen(value.next())?;
                                Ok(ParserNode::new_load(
                                    With::new(inst, next_node.clone()),
                                    rd,
                                    rs1,
                                    With::new(Imm(0), next_node),
                                ))
                            } else {
                                Err(Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    next.ok_or(UnexpectedEOF)?,
                                ))
                            };
                        }
                        Type::Store(inst) => {
                            let rs2 = get_reg(value.next())?;
                            let next = value.next();

                            return if let Ok(imm) = get_imm(next.clone()) {
                                if let Ok(()) = expect_lparen(value.peek().cloned()) {
                                    value.next();
                                    let rs1 = get_reg(value.next())?;
                                    expect_rparen(value.next())?;
                                    Ok(ParserNode::new_store(
                                        With::new(inst, next_node),
                                        rs1,
                                        rs2,
                                        imm,
                                    ))
                                } else {
                                    Ok(ParserNode::new_store(
                                        With::new(inst, next_node.clone()),
                                        With::new(Register::X0, next_node),
                                        rs2,
                                        imm,
                                    ))
                                }
                            } else if let Ok(label) = get_label(next.clone()) {
                                let temp_reg = get_reg(value.next())?;
                                Err(NeedTwoNodes(
                                    Box::new(ParserNode::new_load_addr(
                                        With::new(PseudoType::La, next_node.clone()),
                                        temp_reg.clone(),
                                        label,
                                    )),
                                    Box::new(ParserNode::new_store(
                                        With::new(inst, next_node.clone()),
                                        temp_reg,
                                        rs2,
                                        With::new(Imm(0), next_node),
                                    )),
                                ))
                            } else if let Ok(()) = expect_lparen(next.clone()) {
                                let rs1 = get_reg(value.next())?;
                                expect_rparen(value.next())?;
                                Ok(ParserNode::new_store(
                                    With::new(inst, next_node.clone()),
                                    rs1,
                                    rs2,
                                    With::new(Imm(0), next_node),
                                ))
                            } else {
                                Err(Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    next.ok_or(UnexpectedEOF)?,
                                ))
                            };
                        }
                        Type::Branch(inst) => {
                            let rs1 = get_reg(value.next())?;
                            let rs2 = get_reg(value.next())?;
                            let label = get_label(value.next())?;
                            Ok(ParserNode::new_branch(
                                With::new(inst, next_node),
                                rs1,
                                rs2,
                                label,
                            ))
                        }
                        Type::Ignore(_) => Err(Ignored(next_node)),
                        Type::Basic(inst) => Ok(ParserNode::new_basic(With::new(inst, next_node))),
                        Type::Pseudo(inst) => {
                            // TODO not every pseudo instruction from rars is covered
                            // here.
                            match inst {
                                PseudoType::Ret => {
                                    return Ok(ParserNode::new_jump_link_r(
                                        With::new(JumpLinkRType::Jalr, next_node.clone()),
                                        With::new(Register::X0, next_node.clone()),
                                        With::new(Register::X1, next_node.clone()),
                                        With::new(Imm(0), next_node.clone()),
                                    ))
                                }
                                PseudoType::Mv => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ParserNode::new_arith(
                                        With::new(ArithType::Add, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(Register::X0, next_node.clone()),
                                    ));
                                }
                                PseudoType::Li => {
                                    let rd = get_reg(value.next())?;
                                    let imm = get_imm(value.next())?;
                                    return Ok(ParserNode::new_iarith(
                                        With::new(IArithType::Addi, next_node.clone()),
                                        rd,
                                        With::new(Register::X0, imm.info()),
                                        imm,
                                    ));
                                }
                                PseudoType::La => {
                                    let rd = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_load_addr(
                                        With::new(PseudoType::La, next_node.clone()),
                                        rd,
                                        label,
                                    ));
                                }
                                PseudoType::J | PseudoType::B => {
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_jump_link(
                                        With::new(JumpLinkType::Jal, next_node.clone()),
                                        With::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Jr => {
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ParserNode::new_jump_link_r(
                                        With::new(JumpLinkRType::Jalr, next_node.clone()),
                                        With::new(Register::X0, next_node.clone()),
                                        rs1,
                                        With::new(Imm(0), next_node.clone()),
                                    ));
                                }
                                PseudoType::Beqz => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_branch(
                                        With::new(BranchType::Beq, next_node.clone()),
                                        rs1,
                                        With::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Bnez => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_branch(
                                        With::new(BranchType::Bne, next_node.clone()),
                                        rs1,
                                        With::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Bltz | PseudoType::Bgtz => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_branch(
                                        With::new(BranchType::Blt, next_node.clone()),
                                        rs1,
                                        With::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Neg => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ParserNode::new_arith(
                                        With::new(ArithType::Sub, next_node.clone()),
                                        rd,
                                        With::new(Register::X0, next_node.clone()),
                                        rs1,
                                    ));
                                }
                                PseudoType::Not => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ParserNode::new_iarith(
                                        With::new(IArithType::Xori, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(Imm(-1), next_node.clone()),
                                    ));
                                }
                                PseudoType::Seqz => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ParserNode::new_iarith(
                                        With::new(IArithType::Sltiu, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(Imm(1), next_node.clone()),
                                    ));
                                }
                                PseudoType::Snez => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ParserNode::new_iarith(
                                        With::new(IArithType::Sltiu, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(Imm(0), next_node.clone()),
                                    ));
                                }
                                PseudoType::Nop => {
                                    return Ok(ParserNode::new_iarith(
                                        With::new(IArithType::Addi, next_node.clone()),
                                        With::new(Register::X0, next_node.clone()),
                                        With::new(Register::X0, next_node.clone()),
                                        With::new(Imm(0), next_node.clone()),
                                    ));
                                }
                                PseudoType::Bgez | PseudoType::Blez => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_branch(
                                        With::new(BranchType::Bge, next_node.clone()),
                                        rs1,
                                        With::new(Register::X0, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Sgtz => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ParserNode::new_arith(
                                        With::new(ArithType::Slt, next_node.clone()),
                                        rd,
                                        With::new(Register::X0, next_node.clone()),
                                        rs1,
                                    ));
                                }
                                PseudoType::Sltz => {
                                    let rd = get_reg(value.next())?;
                                    let rs1 = get_reg(value.next())?;
                                    return Ok(ParserNode::new_arith(
                                        With::new(ArithType::Slt, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(Register::X0, next_node.clone()),
                                    ));
                                }
                                PseudoType::Sgez => {
                                    let rs1 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_branch(
                                        With::new(BranchType::Bge, next_node.clone()),
                                        With::new(Register::X0, next_node.clone()),
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Call => {
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_jump_link(
                                        With::new(JumpLinkType::Jal, next_node.clone()),
                                        With::new(Register::X1, next_node.clone()),
                                        label,
                                    ));
                                }
                                PseudoType::Bgt => {
                                    let rs1 = get_reg(value.next())?;
                                    let rs2 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_branch(
                                        With::new(BranchType::Blt, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Ble => {
                                    let rs1 = get_reg(value.next())?;
                                    let rs2 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_branch(
                                        With::new(BranchType::Bge, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Bgtu => {
                                    let rs1 = get_reg(value.next())?;
                                    let rs2 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_branch(
                                        With::new(BranchType::Bltu, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Bleu => {
                                    let rs1 = get_reg(value.next())?;
                                    let rs2 = get_reg(value.next())?;
                                    let label = get_label(value.next())?;
                                    return Ok(ParserNode::new_branch(
                                        With::new(BranchType::Bgeu, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                    ));
                                }
                                PseudoType::Csrci | PseudoType::Csrsi | PseudoType::Csrwi => {
                                    let csr = get_csrimm(value.next())?;
                                    let imm = get_imm(value.next())?;
                                    let inst = match inst {
                                        PseudoType::Csrci => CSRIType::Csrrci,
                                        PseudoType::Csrsi => CSRIType::Csrrsi,
                                        PseudoType::Csrwi => CSRIType::Csrrwi,
                                        _ => return Err(ParseError::UnexpectedError),
                                    };
                                    return Ok(ParserNode::new_csri(
                                        With::new(inst, next_node.clone()),
                                        With::new(Register::X0, next_node.clone()),
                                        csr,
                                        imm,
                                    ));
                                }
                                PseudoType::Csrc | PseudoType::Csrs | PseudoType::Csrw => {
                                    let rs1 = get_reg(value.next())?;
                                    let csr = get_csrimm(value.next())?;
                                    let inst = match inst {
                                        PseudoType::Csrc => CSRType::Csrrc,
                                        PseudoType::Csrs => CSRType::Csrrs,
                                        PseudoType::Csrw => CSRType::Csrrw,
                                        _ => return Err(ParseError::UnexpectedError),
                                    };
                                    return Ok(ParserNode::new_csr(
                                        With::new(inst, next_node.clone()),
                                        With::new(Register::X0, next_node.clone()),
                                        csr,
                                        rs1,
                                    ));
                                }
                                PseudoType::Csrr => {
                                    let rd = get_reg(value.next())?;
                                    let csr = get_csrimm(value.next())?;
                                    return Ok(ParserNode::new_csr(
                                        With::new(CSRType::Csrrs, next_node.clone()),
                                        rd,
                                        csr,
                                        With::new(Register::X0, next_node.clone()),
                                    ));
                                }
                            }
                        }
                    };
                    return node;
                }
                Err(ParseError::Expected(
                    vec![ExpectedType::Inst],
                    next_node.clone(),
                ))
            }
            Token::Label(s) => Ok(ParserNode::new_label(With::new(
                LabelString::from_str(s).map_err(|_| {
                    ParseError::Expected(vec![ExpectedType::Label], next_node.clone())
                })?,
                next_node,
            ))),
            Token::Directive(_) => {
                let node = next_node.clone();
                // skip to the next line
                for token in value.by_ref() {
                    if token == Token::Newline {
                        break;
                    }
                }
                Ok(ParserNode::new_directive(With::new(
                    DirectiveType::Nop,
                    node,
                )))
            }
            Token::Newline => Err(IsNewline(next_node)),
            Token::LParen | Token::RParen | Token::String(_) => {
                Err(ParseError::UnexpectedToken(next_node))
            }
        }
    }
}
