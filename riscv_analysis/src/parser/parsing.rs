use uuid::Uuid;

use crate::parser::inst::{
    ArithType, BranchType, CsrIType, CsrType, IArithType, Inst, JumpLinkRType, JumpLinkType,
    PseudoType, Type,
};
use crate::parser::{DataType, RawToken, Register};
use crate::parser::{DirectiveToken, LexError};
use crate::parser::{DirectiveType, ParserNode};
use crate::parser::{Lexer, TokenType};
use crate::passes::{DiagnosticItem, DiagnosticLocation, Manager, ManagerConfiguration};
use crate::reader::FileReader;
use serde::Deserialize;
use std::iter::Peekable;
use std::str::FromStr;

use super::imm::{CsrImm, Imm};
use super::token::Token;
use super::{ExpectedType, LabelString, LabelStringToken, ParseError, Range, RegisterToken, With};

#[derive(Deserialize, Clone)]
pub struct RVDocument {
    pub uri: String,
    pub text: String,
}

impl ParserNode {
    /// Return a string inside a `.include` directive, if it is a `.include` directive.
    ///
    /// This function returns the token representing the path that is
    /// written in a `.include` directive. If the directive is not a `.include`
    /// directive, it will return `None`. This path is the path used to
    /// read from another file.
    fn get_include_path(&self) -> Option<&With<String>> {
        match self {
            ParserNode::Directive(d) => match &d.dir {
                DirectiveType::Include(path) => Some(path),
                _ => None,
            },
            _ => None,
        }
    }
}

pub trait CanGetURIString: FileReader {
    fn get_uri_string(&self, uuid: Uuid) -> RVDocument;
}
/// Parser for RISC-V assembly
pub struct RVParser<T>
where
    T: FileReader,
{
    lexer_stack: Vec<Peekable<Lexer>>,
    pub reader: T,
}

impl<T: FileReader> RVParser<T> {
    pub fn run(&mut self, base: &str, config: &ManagerConfiguration) -> Vec<DiagnosticItem> {
        let mut diags = Vec::new();
        let parsed = self.parse_from_file(base, false);
        parsed
            .1
            .iter()
            .for_each(|x| diags.push(DiagnosticItem::from(x.clone())));
        let res = Manager::run(parsed.0, config); // TODO make this configurable
        match res {
            Ok(lints) => {
                lints
                    .iter()
                    .for_each(|x| diags.push(DiagnosticItem::from_displayable(x.as_ref())));
            }
            Err(err) => diags.push(DiagnosticItem::from(*err)),
        }
        diags.sort();
        diags
    }

    pub fn run_with_default_config(&mut self, base: &str) -> Vec<DiagnosticItem> {
        self.run(base, &ManagerConfiguration::default())
    }

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
        if let Some(x) = lexer {
            for token in x.by_ref().flatten() {
                if token == TokenType::Newline {
                    break;
                }
            }
        }
    }

    /// Parse files
    ///
    /// This function is responsible for parsing the file. It will continue until no imports are left.
    pub fn parse_from_file(
        &mut self,
        base: &str,
        ignore_imports: bool,
    ) -> (Vec<ParserNode>, Vec<ParseError>) {
        let mut nodes = Vec::new();
        let mut parse_errors = Vec::new();

        // import base lexer
        let lexer = match self.reader.import_file(base, None) {
            Ok(x) => Lexer::new(x.1, x.0),
            Err(e) => {
                parse_errors.push(e.to_parse_error(With::new(base.to_owned(), Token::default())));
                return (nodes, parse_errors);
            }
        };
        let first_uuid = lexer.source_id;
        self.lexer_stack.push(lexer.peekable());

        // Add program entry node
        nodes.push(ParserNode::new_program_entry(
            first_uuid,
            RawToken::new(String::new(), Range::default(), first_uuid),
        ));

        while let Some(l) = self.lexer() {
            let node = ParserNode::try_from(l);

            match node {
                Ok(x) => {
                    if !ignore_imports {
                        if let Some(path) = x.get_include_path() {
                            match self.reader.import_file(path.get(), Some(path.file())) {
                                Ok((new_uuid, new_text)) => {
                                    self.lexer_stack
                                        .push(Lexer::new(new_text, new_uuid).peekable());
                                }
                                Err(error) => {
                                    parse_errors.push(error.to_parse_error(path.clone()));
                                }
                            }
                            continue;
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
                        self.recover_from_parse_error();
                    }
                    LexError::UnknownDirective(y) => {
                        parse_errors.push(ParseError::UnknownDirective(y));
                        self.recover_from_parse_error();
                    }
                    LexError::IgnoredWithWarning(y) | LexError::UnsupportedDirective(y) => {
                        parse_errors.push(ParseError::Unsupported(y));
                        self.recover_from_parse_error();
                    }
                    LexError::IgnoredWithoutWarning => (),
                    LexError::InvalidString(info, err) => {
                        parse_errors.push(ParseError::InvalidString(info, err));
                        self.recover_from_parse_error();
                    }
                },
            }
        }
        (nodes, parse_errors)
    }

    fn lexer(&mut self) -> Option<&mut Peekable<Lexer>> {
        self.lexer_stack.last_mut()
    }
}

impl Token {
    fn as_type<T: TryFrom<Token>, const N: usize>(
        &self,
        errors: [ExpectedType; N],
    ) -> Result<With<T>, LexError> {
        T::try_from(self.clone())
            .map(|x| With::new(x, self.clone()))
            .map_err(|_| LexError::Expected(errors.into(), Box::new(self.clone())))
    }

    fn as_lparen(&self) -> Result<(), LexError> {
        match self.token_type() {
            TokenType::LParen => Ok(()),
            _ => Err(LexError::Expected(
                vec![ExpectedType::LParen],
                Box::new(self.clone()),
            )),
        }
    }

    fn as_rparen(&self) -> Result<(), LexError> {
        match self.token_type() {
            TokenType::RParen => Ok(()),
            _ => Err(LexError::Expected(
                vec![ExpectedType::RParen],
                Box::new(self.clone()),
            )),
        }
    }

    fn as_reg(&self) -> Result<RegisterToken, LexError> {
        self.as_type([ExpectedType::Register])
    }

    fn as_imm(&self) -> Result<With<Imm>, LexError> {
        self.as_type([ExpectedType::Imm])
    }

    fn as_label(&self) -> Result<LabelStringToken, LexError> {
        self.as_type([ExpectedType::Label])
    }

    fn as_csrimm(&self) -> Result<With<CsrImm>, LexError> {
        self.as_type([ExpectedType::CsrImm])
    }

    fn as_string(&self) -> Result<With<String>, LexError> {
        match self.token_type() {
            TokenType::Symbol(s) | TokenType::String(s) => Ok(With::new(s.clone(), self.clone())),
            _ => Err(LexError::Expected(
                vec![ExpectedType::String],
                Box::new(self.clone()),
            )),
        }
    }
}

impl AnnotatedLexer<'_> {
    fn _expect_lparen(&mut self) -> Result<(), LexError> {
        self.get_any()?.as_lparen()
    }

    fn expect_rparen(&mut self) -> Result<(), LexError> {
        self.get_any()?.as_rparen()
    }

    fn get_reg(&mut self) -> Result<RegisterToken, LexError> {
        self.get_any()?.as_reg()
    }

    fn get_imm(&mut self) -> Result<With<Imm>, LexError> {
        self.get_any()?.as_imm()
    }

    fn get_label(&mut self) -> Result<LabelStringToken, LexError> {
        self.get_any()?.as_label()
    }

    fn get_csrimm(&mut self) -> Result<With<CsrImm>, LexError> {
        self.get_any()?.as_csrimm()
    }

    fn get_string(&mut self) -> Result<With<String>, LexError> {
        self.get_any()?.as_string()
    }

    fn get_any(&mut self) -> Result<Token, LexError> {
        let item = self.lexer.next().ok_or(LexError::UnexpectedEOF)?;
        if let Ok(ref item) = item {
            if self.raw_token == RawToken::default() {
                self.raw_token = item.clone().into();
            } else {
                self.raw_token = RawToken::new(
                    format!("{} {}", self.raw_token.raw_text(), item.raw_text()),
                    Range::new(*self.raw_token.range().start(), *item.range().end()),
                    self.raw_token.file(),
                );
            }
        }
        item
    }

    fn peek_any(&mut self) -> Result<Token, LexError> {
        match self.lexer.peek() {
            Some(item) => item.clone(),
            None => Err(LexError::UnexpectedEOF),
        }
    }
}

struct AnnotatedLexer<'a> {
    lexer: &'a mut Peekable<Lexer>,
    raw_token: RawToken,
}
impl TryFrom<&mut Peekable<Lexer>> for ParserNode {
    type Error = LexError;

    // TODO enforce that all "missing" values for With<> resolve to the token
    // of the instruction

    #[allow(clippy::too_many_lines)]
    fn try_from(val: &mut Peekable<Lexer>) -> Result<Self, Self::Error> {
        use LexError::{Expected, IgnoredWithWarning, IsNewline, NeedTwoNodes};

        let mut lex = AnnotatedLexer {
            lexer: val,
            raw_token: RawToken::default(),
        };

        let next_node = lex.get_any()?;
        match next_node.token_type() {
            TokenType::Symbol(s) => {
                if let Ok(inst) = Inst::from_str(s) {
                    let node = match Type::from(&inst) {
                        Type::CsrI(inst) => {
                            let rd = lex.get_reg()?;
                            let csr = lex.get_csrimm()?;
                            let imm = lex.get_imm()?;
                            Ok(ParserNode::new_csri(
                                With::new(inst, next_node),
                                rd,
                                csr,
                                imm,
                                lex.raw_token,
                            ))
                        }
                        Type::Csr(inst) => {
                            let rd = lex.get_reg()?;
                            let csr = lex.get_csrimm()?;
                            let rs1 = lex.get_reg()?;
                            Ok(ParserNode::new_csr(
                                With::new(inst, next_node),
                                rd,
                                csr,
                                rs1,
                                lex.raw_token,
                            ))
                        }
                        Type::UpperArith(inst) => {
                            let rd = lex.get_reg()?;
                            let mut imm = lex.get_imm()?;
                            let new_imm = Imm::new(imm.get().value() << 12);
                            // shift left by 12
                            *imm.get_mut() = new_imm;
                            Ok(ParserNode::new_iarith(
                                With::new(inst, next_node.clone()),
                                rd,
                                With::new(Register::X0, next_node),
                                imm,
                                lex.raw_token,
                            ))
                        }
                        Type::Arith(inst) => {
                            let rd = lex.get_reg()?;
                            let rs1 = lex.get_reg()?;
                            let rs2 = lex.get_reg()?;
                            Ok(ParserNode::new_arith(
                                With::new(inst, next_node),
                                rd,
                                rs1,
                                rs2,
                                lex.raw_token,
                            ))
                        }
                        Type::IArith(inst) => {
                            let rd = lex.get_reg()?;
                            let rs1 = lex.get_reg()?;
                            let imm = lex.get_imm()?;
                            Ok(ParserNode::new_iarith(
                                With::new(inst, next_node),
                                rd,
                                rs1,
                                imm,
                                lex.raw_token,
                            ))
                        }

                        Type::JumpLink(inst) => {
                            let next = lex.get_any()?;

                            return if let Ok(reg) = next.as_reg() {
                                let name = lex.get_label()?;
                                Ok(ParserNode::new_jump_link(
                                    With::new(inst, next_node),
                                    reg,
                                    name,
                                    lex.raw_token,
                                ))
                            } else if let Ok(name) = next.as_label() {
                                Ok(ParserNode::new_jump_link(
                                    With::new(inst, next_node.clone()),
                                    With::new(Register::X1, next_node),
                                    name,
                                    lex.raw_token,
                                ))
                            } else {
                                Err(Expected(
                                    vec![ExpectedType::Register, ExpectedType::Label],
                                    Box::new(next),
                                ))
                            };
                        }
                        Type::JumpLinkR(inst) => {
                            let reg1 = lex.get_reg()?;
                            let next = lex.get_any()?;
                            return if let Ok(rs1) = next.as_reg() {
                                let imm = lex.get_imm()?;
                                Ok(ParserNode::new_jump_link_r(
                                    With::new(inst, next_node),
                                    reg1,
                                    rs1,
                                    imm,
                                    lex.raw_token,
                                ))
                            } else if let Ok(imm) = next.as_imm() {
                                if let Ok(()) = lex.peek_any()?.as_lparen() {
                                    lex.get_any()?;
                                    let rs1 = lex.get_reg()?;
                                    lex.expect_rparen()?;
                                    Ok(ParserNode::new_jump_link_r(
                                        With::new(inst, next_node),
                                        reg1,
                                        rs1,
                                        imm,
                                        lex.raw_token,
                                    ))
                                } else {
                                    Ok(ParserNode::new_jump_link_r(
                                        With::new(inst, next_node.clone()),
                                        With::new(Register::X1, next_node),
                                        reg1,
                                        imm,
                                        lex.raw_token,
                                    ))
                                }
                            } else if let Ok(()) = next.as_lparen() {
                                let rs1 = lex.get_reg()?;
                                lex.expect_rparen()?;
                                Ok(ParserNode::new_jump_link_r(
                                    With::new(inst, next_node.clone()),
                                    reg1,
                                    rs1,
                                    With::new(Imm::new(0), next_node),
                                    lex.raw_token,
                                ))
                            } else {
                                Ok(ParserNode::new_jump_link_r(
                                    With::new(inst, next_node.clone()),
                                    With::new(Register::X1, next_node.clone()),
                                    reg1,
                                    With::new(Imm::new(0), next_node),
                                    lex.raw_token,
                                ))
                            };
                        }
                        Type::Load(inst) => {
                            let rd = lex.get_reg()?;
                            let next = lex.get_any()?;
                            return if let Ok(imm) = next.as_imm() {
                                if let Ok(()) = lex.peek_any()?.as_lparen() {
                                    lex.get_any()?;
                                    let rs1 = lex.get_reg()?;
                                    lex.expect_rparen()?;
                                    Ok(ParserNode::new_load(
                                        With::new(inst, next_node),
                                        rd,
                                        rs1,
                                        imm,
                                        lex.raw_token,
                                    ))
                                } else {
                                    Ok(ParserNode::new_load(
                                        With::new(inst, next_node.clone()),
                                        rd,
                                        With::new(Register::X0, next_node),
                                        imm,
                                        lex.raw_token,
                                    ))
                                }
                            } else if let Ok(label) = next.as_label() {
                                Err(NeedTwoNodes(
                                    Box::new(ParserNode::new_load_addr(
                                        With::new(PseudoType::La, next_node.clone()),
                                        rd.clone(),
                                        label,
                                        lex.raw_token.clone(),
                                    )),
                                    Box::new(ParserNode::new_load(
                                        With::new(inst, next_node.clone()),
                                        rd.clone(),
                                        rd,
                                        With::new(Imm::new(0), next_node),
                                        lex.raw_token,
                                    )),
                                ))
                            } else if let Ok(()) = next.as_lparen() {
                                let rs1 = lex.get_reg()?;
                                lex.expect_rparen()?;
                                Ok(ParserNode::new_load(
                                    With::new(inst, next_node.clone()),
                                    rd,
                                    rs1,
                                    With::new(Imm::new(0), next_node),
                                    lex.raw_token,
                                ))
                            } else {
                                Err(Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    Box::new(next),
                                ))
                            };
                        }
                        Type::Store(inst) => {
                            let rs2 = lex.get_reg()?;
                            let next = lex.get_any()?;

                            return if let Ok(imm) = next.as_imm() {
                                if let Ok(()) = lex.peek_any()?.as_lparen() {
                                    lex.get_any()?;
                                    let rs1 = lex.get_reg()?;
                                    lex.expect_rparen()?;
                                    Ok(ParserNode::new_store(
                                        With::new(inst, next_node),
                                        rs1,
                                        rs2,
                                        imm,
                                        lex.raw_token,
                                    ))
                                } else if let Ok(tmp) = lex.peek_any()?.as_reg() {
                                    lex.get_any()?;
                                    Err(LexError::NeedTwoNodes(
                                        Box::new(ParserNode::new_iarith(
                                            With::new(IArithType::Addi, next_node.clone()),
                                            tmp.clone(),
                                            With::new(Register::X0, next_node.clone()),
                                            imm,
                                            lex.raw_token.clone(),
                                        )),
                                        Box::new(ParserNode::new_store(
                                            With::new(inst, next_node.clone()),
                                            tmp,
                                            rs2,
                                            With::new(Imm::new(0), next_node),
                                            lex.raw_token,
                                        )),
                                    ))
                                } else {
                                    Ok(ParserNode::new_store(
                                        With::new(inst, next_node.clone()),
                                        With::new(Register::X0, next_node),
                                        rs2,
                                        imm,
                                        lex.raw_token,
                                    ))
                                }
                            } else if let Ok(label) = next.as_label() {
                                let temp_reg = lex.get_reg()?;
                                Err(NeedTwoNodes(
                                    Box::new(ParserNode::new_load_addr(
                                        With::new(PseudoType::La, next_node.clone()),
                                        temp_reg.clone(),
                                        label,
                                        lex.raw_token.clone(),
                                    )),
                                    Box::new(ParserNode::new_store(
                                        With::new(inst, next_node.clone()),
                                        temp_reg,
                                        rs2,
                                        With::new(Imm::new(0), next_node),
                                        lex.raw_token,
                                    )),
                                ))
                            } else if let Ok(()) = next.as_lparen() {
                                let rs1 = lex.get_reg()?;
                                lex.expect_rparen()?;
                                Ok(ParserNode::new_store(
                                    With::new(inst, next_node.clone()),
                                    rs1,
                                    rs2,
                                    With::new(Imm::new(0), next_node),
                                    lex.raw_token,
                                ))
                            } else {
                                Err(Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    Box::new(next),
                                ))
                            };
                        }
                        Type::Branch(inst) => {
                            let rs1 = lex.get_reg()?;
                            let rs2 = lex.get_reg()?;
                            let label = lex.get_label()?;
                            Ok(ParserNode::new_branch(
                                With::new(inst, next_node),
                                rs1,
                                rs2,
                                label,
                                lex.raw_token,
                            ))
                        }
                        Type::Ignore(_) => Err(IgnoredWithWarning(Box::new(next_node))),
                        Type::Basic(inst) => Ok(ParserNode::new_basic(
                            With::new(inst, next_node),
                            lex.raw_token,
                        )),
                        Type::Pseudo(inst) => match inst {
                            PseudoType::Ret => {
                                return Ok(ParserNode::new_jump_link_r(
                                    With::new(JumpLinkRType::Jalr, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    With::new(Register::X1, next_node.clone()),
                                    With::new(Imm::new(0), next_node.clone()),
                                    lex.raw_token,
                                ))
                            }
                            PseudoType::Mv => {
                                let rd = lex.get_reg()?;
                                let rs1 = lex.get_reg()?;
                                return Ok(ParserNode::new_arith(
                                    With::new(ArithType::Add, next_node.clone()),
                                    rd,
                                    rs1,
                                    With::new(Register::X0, next_node.clone()),
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Li => {
                                let rd = lex.get_reg()?;
                                let imm = lex.get_imm()?;
                                return Ok(ParserNode::new_iarith(
                                    With::new(IArithType::Addi, next_node.clone()),
                                    rd,
                                    With::new(Register::X0, imm.token().clone()),
                                    imm,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::La => {
                                let rd = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_load_addr(
                                    With::new(PseudoType::La, next_node.clone()),
                                    rd,
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::J | PseudoType::B => {
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_jump_link(
                                    With::new(JumpLinkType::Jal, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Jr => {
                                let rs1 = lex.get_reg()?;
                                return Ok(ParserNode::new_jump_link_r(
                                    With::new(JumpLinkRType::Jalr, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    rs1,
                                    With::new(Imm::new(0), next_node.clone()),
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Beqz => {
                                let rs1 = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_branch(
                                    With::new(BranchType::Beq, next_node.clone()),
                                    rs1,
                                    With::new(Register::X0, next_node.clone()),
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Bnez => {
                                let rs1 = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_branch(
                                    With::new(BranchType::Bne, next_node.clone()),
                                    rs1,
                                    With::new(Register::X0, next_node.clone()),
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Bltz | PseudoType::Bgtz => {
                                let rs1 = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_branch(
                                    With::new(BranchType::Blt, next_node.clone()),
                                    rs1,
                                    With::new(Register::X0, next_node.clone()),
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Neg => {
                                let rd = lex.get_reg()?;
                                let rs1 = lex.get_reg()?;
                                return Ok(ParserNode::new_arith(
                                    With::new(ArithType::Sub, next_node.clone()),
                                    rd,
                                    With::new(Register::X0, next_node.clone()),
                                    rs1,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Not => {
                                let rd = lex.get_reg()?;
                                let rs1 = lex.get_reg()?;
                                return Ok(ParserNode::new_iarith(
                                    With::new(IArithType::Xori, next_node.clone()),
                                    rd,
                                    rs1,
                                    With::new(Imm::new(-1), next_node.clone()),
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Seqz => {
                                let rd = lex.get_reg()?;
                                let rs1 = lex.get_reg()?;
                                return Ok(ParserNode::new_iarith(
                                    With::new(IArithType::Sltiu, next_node.clone()),
                                    rd,
                                    rs1,
                                    With::new(Imm::new(1), next_node.clone()),
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Snez => {
                                let rd = lex.get_reg()?;
                                let rs1 = lex.get_reg()?;
                                return Ok(ParserNode::new_iarith(
                                    With::new(IArithType::Sltiu, next_node.clone()),
                                    rd,
                                    rs1,
                                    With::new(Imm::new(0), next_node.clone()),
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Nop => {
                                return Ok(ParserNode::new_iarith(
                                    With::new(IArithType::Addi, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    With::new(Imm::new(0), next_node.clone()),
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Bgez | PseudoType::Blez => {
                                let rs1 = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_branch(
                                    With::new(BranchType::Bge, next_node.clone()),
                                    rs1,
                                    With::new(Register::X0, next_node.clone()),
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Sgtz => {
                                let rd = lex.get_reg()?;
                                let rs1 = lex.get_reg()?;
                                return Ok(ParserNode::new_arith(
                                    With::new(ArithType::Slt, next_node.clone()),
                                    rd,
                                    With::new(Register::X0, next_node.clone()),
                                    rs1,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Sltz => {
                                let rd = lex.get_reg()?;
                                let rs1 = lex.get_reg()?;
                                return Ok(ParserNode::new_arith(
                                    With::new(ArithType::Slt, next_node.clone()),
                                    rd,
                                    rs1,
                                    With::new(Register::X0, next_node.clone()),
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Sgez => {
                                let rs1 = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_branch(
                                    With::new(BranchType::Bge, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    rs1,
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Call => {
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_jump_link(
                                    With::new(JumpLinkType::Jal, next_node.clone()),
                                    With::new(Register::X1, next_node.clone()),
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Bgt => {
                                let rs1 = lex.get_reg()?;
                                let rs2 = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_branch(
                                    With::new(BranchType::Blt, next_node.clone()),
                                    rs2,
                                    rs1,
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Ble => {
                                let rs1 = lex.get_reg()?;
                                let rs2 = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_branch(
                                    With::new(BranchType::Bge, next_node.clone()),
                                    rs2,
                                    rs1,
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Bgtu => {
                                let rs1 = lex.get_reg()?;
                                let rs2 = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_branch(
                                    With::new(BranchType::Bltu, next_node.clone()),
                                    rs2,
                                    rs1,
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Bleu => {
                                let rs1 = lex.get_reg()?;
                                let rs2 = lex.get_reg()?;
                                let label = lex.get_label()?;
                                return Ok(ParserNode::new_branch(
                                    With::new(BranchType::Bgeu, next_node.clone()),
                                    rs2,
                                    rs1,
                                    label,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Csrci | PseudoType::Csrsi | PseudoType::Csrwi => {
                                let csr = lex.get_csrimm()?;
                                let imm = lex.get_imm()?;
                                let inst = match inst {
                                    PseudoType::Csrci => CsrIType::Csrrci,
                                    PseudoType::Csrsi => CsrIType::Csrrsi,
                                    PseudoType::Csrwi => CsrIType::Csrrwi,
                                    _ => {
                                        return Err(LexError::UnexpectedError(Box::new(
                                            next_node.clone(),
                                        )))
                                    }
                                };
                                return Ok(ParserNode::new_csri(
                                    With::new(inst, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    csr,
                                    imm,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Csrc | PseudoType::Csrs | PseudoType::Csrw => {
                                let rs1 = lex.get_reg()?;
                                let csr = lex.get_csrimm()?;
                                let inst = match inst {
                                    PseudoType::Csrc => CsrType::Csrrc,
                                    PseudoType::Csrs => CsrType::Csrrs,
                                    PseudoType::Csrw => CsrType::Csrrw,
                                    _ => {
                                        return Err(LexError::UnexpectedError(Box::new(
                                            next_node.clone(),
                                        )))
                                    }
                                };
                                return Ok(ParserNode::new_csr(
                                    With::new(inst, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    csr,
                                    rs1,
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Csrr => {
                                let rd = lex.get_reg()?;
                                let csr = lex.get_csrimm()?;
                                return Ok(ParserNode::new_csr(
                                    With::new(CsrType::Csrrs, next_node.clone()),
                                    rd,
                                    csr,
                                    With::new(Register::X0, next_node.clone()),
                                    lex.raw_token,
                                ));
                            }
                        },
                    };
                    return node;
                }
                Err(LexError::Expected(
                    vec![ExpectedType::Inst],
                    Box::new(next_node.clone()),
                ))
            }
            TokenType::Label(s) => Ok(ParserNode::new_label(
                With::new(
                    LabelString::from_str(s).map_err(|()| {
                        LexError::Expected(vec![ExpectedType::Label], Box::new(next_node.clone()))
                    })?,
                    next_node,
                ),
                lex.raw_token,
            )),
            TokenType::Directive(dir) => {
                if let Ok(directive) = DirectiveToken::from_str(dir) {
                    match directive {
                        DirectiveToken::Align => {
                            let imm = lex.get_imm()?;
                            Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Align(imm),
                                lex.raw_token,
                            ))
                        }
                        DirectiveToken::Ascii => {
                            let string = lex.get_string()?;
                            Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Ascii {
                                    text: string,
                                    null_term: false,
                                },
                                lex.raw_token,
                            ))
                        }
                        DirectiveToken::Asciz | DirectiveToken::String => {
                            let string = lex.get_string()?;
                            Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Ascii {
                                    text: string,
                                    null_term: true,
                                },
                                lex.raw_token,
                            ))
                        }
                        DirectiveToken::Byte
                        | DirectiveToken::Double
                        | DirectiveToken::Dword
                        | DirectiveToken::Float
                        | DirectiveToken::Word
                        | DirectiveToken::Half => {
                            let data_type = match directive {
                                DirectiveToken::Byte => DataType::Byte,
                                DirectiveToken::Double => DataType::Double,
                                DirectiveToken::Dword => DataType::Dword,
                                DirectiveToken::Float => DataType::Float,
                                DirectiveToken::Word => DataType::Word,
                                DirectiveToken::Half => DataType::Half,
                                _ => return Err(LexError::UnexpectedError(Box::new(next_node))),
                            };

                            // keep looping through values until immediate or nl is
                            // not found
                            let mut values = Vec::new();
                            loop {
                                let next = lex.peek_any()?;
                                if let TokenType::Newline = next.token_type() {
                                    // consume newline
                                    lex.get_any()?;
                                } else if let Ok(imm) = next.as_imm() {
                                    // try to get immediate
                                    lex.get_any()?;
                                    values.push(imm);
                                } else {
                                    break;
                                }
                            }

                            Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Data(data_type, values),
                                lex.raw_token,
                            ))
                        }
                        DirectiveToken::Data => Ok(ParserNode::new_directive(
                            With::new(directive, next_node.clone()),
                            DirectiveType::DataSection,
                            lex.raw_token,
                        )),
                        DirectiveToken::Macro => {
                            // macros are unsupported
                            // we will just ignore them until the we reach endmacro
                            loop {
                                let next = lex.get_any()?;
                                if let TokenType::Directive(dir2) = next.token_type() {
                                    if let Ok(new_dir) = DirectiveToken::from_str(dir2) {
                                        if new_dir == DirectiveToken::EndMacro {
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(LexError::IgnoredWithWarning(Box::new(next_node)))
                        }
                        DirectiveToken::EndMacro => {
                            Err(LexError::IgnoredWithWarning(Box::new(next_node)))
                        }
                        DirectiveToken::Section
                        | DirectiveToken::Extern
                        | DirectiveToken::Eqv
                        | DirectiveToken::Global
                        | DirectiveToken::Globl => {
                            Err(LexError::UnsupportedDirective(Box::new(next_node)))
                        }
                        DirectiveToken::Include => {
                            let filename = lex.get_string()?;
                            Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Include(filename),
                                lex.raw_token,
                            ))
                        }
                        DirectiveToken::Space => {
                            let imm = lex.get_imm()?;
                            Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Space(imm),
                                lex.raw_token,
                            ))
                        }
                        DirectiveToken::Text => Ok(ParserNode::new_directive(
                            With::new(directive, next_node.clone()),
                            DirectiveType::TextSection,
                            lex.raw_token,
                        )),
                    }
                } else {
                    Err(LexError::UnknownDirective(Box::new(next_node.clone())))
                }
            }
            TokenType::Newline => Err(IsNewline(Box::new(next_node))),
            TokenType::LParen | TokenType::RParen | TokenType::String(_) | TokenType::Char(_) => {
                Err(LexError::UnexpectedToken(Box::new(next_node)))
            }
            // Skip comment token
            TokenType::Comment(_) => Err(LexError::IgnoredWithoutWarning),
        }
    }
}
