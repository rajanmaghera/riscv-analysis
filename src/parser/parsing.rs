use crate::parser::inst::{
    ArithType, BranchType, CSRIType, CSRType, IArithType, Inst, JumpLinkRType, JumpLinkType,
    PseudoType, Type,
};
use crate::parser::token::With;
use crate::parser::Register;
use crate::parser::{DirectiveToken, LexError};
use crate::parser::{DirectiveType, ParserNode};
use crate::parser::{Lexer, Token};
use crate::reader::{FileReader, FileReaderError};
use std::iter::Peekable;
use std::str::FromStr;

use super::imm::{CSRImm, Imm};
use super::token::Info;
use super::{ExpectedType, LabelString, ParseError};

/// Parser for RISC-V assembly
pub struct RVParser<T>
where
    T: FileReader,
{
    lexer_stack: Vec<Peekable<Lexer>>,
    pub reader: T,
}

impl<T: FileReader> RVParser<T> {
    pub fn new(reader: T) -> RVParser<T> {
        RVParser {
            lexer_stack: Vec::new(),
            reader,
        }
    }

    /// Skip the rest of the line
    ///
    /// This is used to recover from parse errors. If there is a parse error,
    /// we will skip the rest of the line and try to parse the next line.
    fn recover_from_parse_error(&mut self) {
        let lexer = self.lexer();
        match lexer {
            Some(x) => {
                for token in x.by_ref() {
                    if token == Token::Newline {
                        break;
                    }
                }
            }
            None => {}
        }
    }

    /// Parse files
    ///
    /// This function is responsible for parsing the file. It will continue until no imports are left.
    pub fn parse(
        &mut self,
        base: &str,
        ignore_imports: bool,
    ) -> (Vec<ParserNode>, Vec<ParseError>) {
        let mut nodes = Vec::new();
        let mut parse_errors = Vec::new();

        // import base lexer
        let lexer = match self.reader.import_file(&base, None) {
            Ok(x) => x,
            Err(_) => {
                parse_errors.push(ParseError::FileNotFound(With::new(
                    base.to_owned(),
                    Info::default(),
                )));
                return (nodes, parse_errors);
            }
        };
        self.lexer_stack.push(lexer.1);

        // Add program entry node
        nodes.push(ParserNode::new_program_entry(lexer.0));

        while let Some(lexer) = self.lexer() {
            let node = ParserNode::try_from(lexer);

            match node {
                Ok(x) => {
                    if !ignore_imports {
                        if let ParserNode::Directive(directive) = &x {
                            if let DirectiveType::Include(path) = &directive.dir {
                                let lexer = self
                                    .reader
                                    .import_file(path.data.as_str(), Some(directive.token.file));
                                match lexer {
                                    Ok(x) => {
                                        self.lexer_stack.push(x.1);
                                    }
                                    Err(x) => match x {
                                        FileReaderError::IOError(_) => {
                                            parse_errors
                                                .push(ParseError::FileNotFound(path.clone()));
                                        }
                                        FileReaderError::InternalFileNotFound => {
                                            parse_errors
                                                .push(ParseError::UnexpectedError(path.info()));
                                        }
                                        FileReaderError::FileAlreadyRead(_) => {
                                            parse_errors
                                                .push(ParseError::CyclicDependency(path.info()));
                                        }
                                        FileReaderError::Unexpected => {
                                            parse_errors
                                                .push(ParseError::UnexpectedError(path.info()));
                                        }
                                        FileReaderError::InvalidPath => {
                                            parse_errors
                                                .push(ParseError::FileNotFound(path.clone()));
                                        }
                                    },
                                }
                                continue;
                            }
                        }
                    }
                    nodes.push(x);
                }
                Err(x) => match x {
                    LexError::Expected(ex, got) => {
                        parse_errors.push(ParseError::Expected(ex, got));
                        self.recover_from_parse_error();
                    }
                    LexError::IsNewline(_) => {}
                    LexError::Ignored(y) => {
                        parse_errors.push(ParseError::Unsupported(y));
                        self.recover_from_parse_error();
                    }
                    LexError::UnexpectedToken(got) => {
                        parse_errors.push(ParseError::UnexpectedToken(got));
                        self.recover_from_parse_error();
                    }
                    LexError::UnexpectedEOF => {
                        self.lexer_stack.pop();
                    }
                    LexError::NeedTwoNodes(n1, n2) => {
                        nodes.push(*n1);
                        nodes.push(*n2);
                    }
                    LexError::UnexpectedError(x) => {
                        parse_errors.push(ParseError::UnexpectedError(x));
                        // TODO determine how to recover from this and where to markup
                        self.recover_from_parse_error();
                    }
                    LexError::UnknownDirective(y) => {
                        parse_errors.push(ParseError::UnknownDirective(y));
                        self.recover_from_parse_error();
                    }
                },
            }
        }
        println!("parse errors: {:?}", parse_errors);
        (nodes, parse_errors)
    }

    fn lexer(&mut self) -> Option<&mut Peekable<Lexer>> {
        let item = &mut self.lexer_stack;
        item.last_mut()
    }
}

fn expect_lparen(value: Option<Info>) -> Result<(), LexError> {
    let v = value.ok_or(LexError::UnexpectedEOF)?;
    match v.token {
        Token::LParen => Ok(()),
        _ => Err(LexError::Expected(vec![ExpectedType::LParen], v)),
    }
}

fn expect_rparen(value: Option<Info>) -> Result<(), LexError> {
    let v = value.ok_or(LexError::UnexpectedEOF)?;
    match v.token {
        Token::RParen => Ok(()),
        _ => Err(LexError::Expected(vec![ExpectedType::RParen], v)),
    }
}

fn get_reg(value: Option<Info>) -> Result<With<Register>, LexError> {
    let v = value.ok_or(LexError::UnexpectedEOF)?;
    With::<Register>::try_from(v.clone())
        .map_err(|_| LexError::Expected(vec![ExpectedType::Register], v))
}

fn get_imm(value: Option<Info>) -> Result<With<Imm>, LexError> {
    let v = value.ok_or(LexError::UnexpectedEOF)?;
    With::<Imm>::try_from(v.clone()).map_err(|_| LexError::Expected(vec![ExpectedType::Imm], v))
}

fn get_label(value: Option<Info>) -> Result<With<LabelString>, LexError> {
    let v = value.ok_or(LexError::UnexpectedEOF)?;
    With::<LabelString>::try_from(v.clone())
        .map_err(|_| LexError::Expected(vec![ExpectedType::Label], v))
}

fn get_csrimm(value: Option<Info>) -> Result<With<CSRImm>, LexError> {
    let v = value.ok_or(LexError::UnexpectedEOF)?;
    With::<CSRImm>::try_from(v.clone())
        .map_err(|_| LexError::Expected(vec![ExpectedType::CSRImm], v))
}

fn get_string(value: Option<Info>) -> Result<With<String>, LexError> {
    let v = value.ok_or(LexError::UnexpectedEOF)?;
    With::<String>::try_from(v.clone())
        .map_err(|_| LexError::Expected(vec![ExpectedType::String], v))
}

impl TryFrom<&mut Peekable<Lexer>> for ParserNode {
    type Error = LexError;

    // TODO enforce that all "missing" values for With<> resolve to the token
    // of the instruction

    #[allow(clippy::too_many_lines)]
    fn try_from(value: &mut Peekable<Lexer>) -> Result<Self, Self::Error> {
        use LexError::{Expected, Ignored, IsNewline, NeedTwoNodes, UnexpectedEOF};
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
                        Type::Pseudo(inst) => match inst {
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
                                    _ => return Err(LexError::UnexpectedError(next_node.clone())),
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
                                    _ => return Err(LexError::UnexpectedError(next_node.clone())),
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
                        },
                    };
                    return node;
                }
                Err(LexError::Expected(
                    vec![ExpectedType::Inst],
                    next_node.clone(),
                ))
            }
            Token::Label(s) => Ok(ParserNode::new_label(With::new(
                LabelString::from_str(s).map_err(|_| {
                    LexError::Expected(vec![ExpectedType::Label], next_node.clone())
                })?,
                next_node,
            ))),
            Token::Directive(dir) => {
                if let Ok(directive) = DirectiveToken::from_str(dir) {
                    match directive {
                        DirectiveToken::Align => todo!(),
                        DirectiveToken::Ascii => todo!(),
                        DirectiveToken::Asciz => todo!(),
                        DirectiveToken::Byte => todo!(),
                        DirectiveToken::Data => todo!(),
                        DirectiveToken::Double => todo!(),
                        DirectiveToken::Dword => todo!(),
                        DirectiveToken::EndMacro => todo!(),
                        DirectiveToken::Eqv => todo!(),
                        DirectiveToken::Extern => todo!(),
                        DirectiveToken::Float => todo!(),
                        DirectiveToken::Global | DirectiveToken::Globl => todo!(),
                        DirectiveToken::Half => todo!(),
                        DirectiveToken::Include => {
                            let filename = get_string(value.next())?;
                            return Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Include(filename),
                            ));
                        }
                        DirectiveToken::Macro => todo!(),
                        DirectiveToken::Section => todo!(),
                        DirectiveToken::Space => todo!(),
                        DirectiveToken::String => todo!(),
                        DirectiveToken::Text => todo!(),
                        DirectiveToken::Word => todo!(),
                    }
                } else {
                    return Err(LexError::UnknownDirective(next_node.clone()));
                }
            }
            Token::Newline => Err(IsNewline(next_node)),
            Token::LParen | Token::RParen | Token::String(_) => {
                Err(LexError::UnexpectedToken(next_node))
            }
        }
    }
}
