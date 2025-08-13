use std::collections::{HashSet, VecDeque};
use uuid::Uuid;

use super::imm::{CsrImm, Imm};
use super::token::RVToken;
use super::{ExpectedType, LabelString, LabelStringToken, ParseError, RegisterToken, With};
use crate::cfg::Segment;
use crate::parser::inst::{
    ArithType, BranchType, CsrIType, CsrType, IArithType, JumpLinkRType, JumpLinkType, PseudoType,
    RVInst, RVInstType,
};
use crate::parser::RVInstructionNode;
use crate::parser::{DirectiveToken, LexError};
use crate::parser::{RVLexer, TokenType};
use crate::parser::{RVRegister, RawToken};
use crate::passes::{DiagnosticItem, DiagnosticLocation, Manager};
use crate::reader::{FileReader, FileReaderError};
use itertools::Itertools;
use serde::Deserialize;
use std::mem;
use std::str::FromStr;

#[derive(Deserialize, Clone)]
pub struct RVDocument {
    pub uri: String,
    pub text: String,
}

pub trait CanGetURIString: FileReader {
    fn get_uri_string(&self, uuid: Uuid) -> RVDocument;
}

/// The behaviour that the parser should
/// find the program entry with
#[derive(Deserialize, Clone)]
pub enum ProgramEntryType {
    /// Look for a label with this name. If it isn't found,
    /// return an error.
    LookForLabel(With<LabelString>),
    /// Use the first instruction.
    FirstInstruction,
    /// Do not have any program entry
    None,
}

/// Parser for RISC-V assembly
pub struct RVFileParser<T>
where
    T: FileReader,
{
    lexer_stack: Vec<RVParser>,
    pub reader: T,
}

#[derive(Clone, Debug, Default)]
pub struct RVParserOutput {
    pub nodes: Vec<RVInstructionNode>,
    pub all_defined_labels: HashSet<LabelStringToken>,
    pub errors: Vec<ParseError>,
    pub extra_labels: HashSet<LabelStringToken>,
    pub include_strings: HashSet<With<String>>,
}

impl<T: FileReader> RVFileParser<T> {
    pub fn run(
        &mut self,
        base: &str,
        program_entry: &ProgramEntryType,
    ) -> Result<Vec<DiagnosticItem>, FileReaderError> {
        let parsed = self.parse_from_file(base, false)?;
        let mut diags = parsed
            .errors
            .iter()
            .cloned()
            .map_into()
            .collect::<Vec<DiagnosticItem>>();

        let res = Manager::run(&parsed, program_entry);
        match res {
            Ok(lints) => diags.extend(lints.iter().map(DiagnosticItem::from_displayable)),
            Err(err) => diags.push(DiagnosticItem::from(*err)),
        }
        diags.sort();
        Ok(diags)
    }

    pub fn new(reader: T) -> RVFileParser<T> {
        RVFileParser {
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
            for token in x.lexer.by_ref().flatten() {
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
    ) -> Result<RVParserOutput, FileReaderError> {
        let mut nodes = Vec::new();
        let mut all_defined_labels = HashSet::new();
        let mut errors = Vec::new();
        let mut extra_labels = HashSet::new();
        let mut include_strings = HashSet::new();

        // import base lexer
        let (id, source) = self.reader.import_file(base, None)?;
        let lexer = RVLexer::new(source, id);
        self.lexer_stack.push(RVParser::new(lexer));

        while let Some(l) = self.lexer() {
            let node = RVInstructionNode::try_from(l);

            match node {
                Ok(x) => {
                    nodes.push(x);
                }
                Err(x) => match x {
                    LexError::Expected(ex, got) => {
                        errors.push(ParseError::Expected(ex, got));
                        self.recover_from_parse_error();
                    }
                    LexError::UnexpectedToken(got) => {
                        errors.push(ParseError::UnexpectedToken(got));
                        self.recover_from_parse_error();
                    }
                    LexError::UnexpectedEOF => {
                        if let Some(lex) = self.lexer_stack.pop() {
                            all_defined_labels.extend(lex.all_defined_labels);
                        }
                    }
                    LexError::UnexpectedError(x) => {
                        errors.push(ParseError::UnexpectedError(x));
                        self.recover_from_parse_error();
                    }
                    LexError::UnknownDirective(y) => {
                        errors.push(ParseError::UnknownDirective(y));
                        self.recover_from_parse_error();
                    }
                    LexError::IgnoredWithWarning(y) | LexError::UnsupportedDirective(y) => {
                        errors.push(ParseError::Unsupported(y));
                        self.recover_from_parse_error();
                    }
                    LexError::InvalidString(info, err) => {
                        errors.push(ParseError::InvalidString(info, err));
                        self.recover_from_parse_error();
                    }
                    LexError::GlobalDef(label) => {
                        extra_labels.insert(*label);
                    }
                    LexError::IncludeFile(path) => {
                        include_strings.insert(*path.clone());
                        if !ignore_imports {
                            match self.reader.import_file(path.get(), Some(path.file())) {
                                Ok((new_uuid, new_text)) => {
                                    self.lexer_stack
                                        .push(RVParser::new(RVLexer::new(new_text, new_uuid)));
                                }
                                Err(error) => {
                                    errors.push(error.to_parse_error(*path));
                                }
                            }
                        }
                    }
                    LexError::IgnoredWithoutWarning | LexError::IsNewline(_) => (),
                },
            }
        }
        Ok(RVParserOutput {
            nodes,
            all_defined_labels,
            errors,
            extra_labels,
            include_strings,
        })
    }

    fn lexer(&mut self) -> Option<&mut RVParser> {
        self.lexer_stack.last_mut()
    }
}

impl TryFrom<RVToken> for String {
    type Error = ();

    fn try_from(value: RVToken) -> Result<Self, Self::Error> {
        match value.token_type() {
            TokenType::Symbol(s) => Ok(s.clone()),
            _ => Err(()),
        }
    }
}

impl RVToken {
    fn as_type<T: TryFrom<RVToken>, const N: usize>(
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

    fn as_symbol(&self) -> Result<With<String>, LexError> {
        self.as_type([ExpectedType::Symbol])
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

impl RVParser {
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

    fn get_symbol(&mut self) -> Result<With<String>, LexError> {
        self.get_any()?.as_symbol()
    }

    fn get_csrimm(&mut self) -> Result<With<CsrImm>, LexError> {
        self.get_any()?.as_csrimm()
    }

    fn get_string(&mut self) -> Result<With<String>, LexError> {
        self.get_any()?.as_string()
    }

    fn lex_next(&mut self) -> Result<RVToken, LexError> {
        let item = self.lexer.next().ok_or(LexError::UnexpectedEOF)??;
        if let Some(ref mut raw_token) = self.raw_token.as_mut() {
            self.lexer.extend_raw_token(raw_token, &item.clone().into());
        } else {
            self.raw_token = Some(item.clone().into());
        }
        Ok(item)
    }

    fn get_any(&mut self) -> Result<RVToken, LexError> {
        let item = self.lex_next()?;
        if item.token_type() == &TokenType::PercentHigh
            || item.token_type() == &TokenType::PercentLow
        {
            self.lex_next()?.as_lparen()?;
            self.lex_next()?.as_label()?;
            let mut end = self.lex_next()?;
            if let TokenType::Plus(_) = end.token_type() {
                end = self.lex_next()?;
            }
            end.as_rparen()?;
            let mut token = item.raw_token().clone();
            self.lexer.extend_raw_token(&mut token, end.raw_token());
            Ok(RVToken::new(
                item.token_type().clone(),
                token.raw_text(),
                token.range(),
                token.file(),
            ))
        } else {
            Ok(item)
        }
    }

    fn peek_any(&mut self) -> Result<RVToken, LexError> {
        self.lexer.peek().ok_or(LexError::UnexpectedEOF)?
    }

    fn take_raw_token(&mut self) -> Result<RawToken, LexError> {
        self.raw_token.take().ok_or(LexError::UnexpectedEOF)
    }
}

struct RVParser {
    lexer: RVLexer,
    all_defined_labels: HashSet<LabelStringToken>,
    raw_token: Option<RawToken>,
    current_segment: Segment,
    current_labels: HashSet<LabelStringToken>,
    queue: VecDeque<RVInstructionNode>,
}

impl RVParser {
    pub fn new(lexer: RVLexer) -> RVParser {
        Self {
            lexer,
            all_defined_labels: HashSet::new(),
            raw_token: None,
            current_segment: Segment::Text,
            current_labels: HashSet::new(),
            queue: VecDeque::new(),
        }
    }
}

impl TryFrom<&mut RVParser> for RVInstructionNode {
    type Error = LexError;

    // TODO enforce that all "missing" values for With<> resolve to the token
    // of the instruction

    // TODO: switch try_from to iterator that returns a result

    #[allow(clippy::too_many_lines)]
    fn try_from(parser: &mut RVParser) -> Result<Self, Self::Error> {
        if let Some(node) = parser.queue.pop_front() {
            return Ok(node);
        }

        parser.raw_token = None;
        let next_node = parser.get_any()?;
        match next_node.token_type() {
            TokenType::Symbol(s) => {
                if let Ok(inst) = RVInst::from_str(s) {
                    let node = match RVInstType::from(&inst) {
                        RVInstType::CsrI(inst) => {
                            let rd = parser.get_reg()?;
                            let csr = parser.get_csrimm()?;
                            let imm = parser.get_imm()?;
                            let raw_token = parser.take_raw_token()?;
                            Ok(RVInstructionNode::new_csri(
                                With::new(inst, next_node),
                                rd,
                                csr,
                                imm,
                                raw_token,
                                parser.current_segment,
                                mem::take(&mut parser.current_labels),
                            ))
                        }
                        RVInstType::Csr(inst) => {
                            let rd = parser.get_reg()?;
                            let csr = parser.get_csrimm()?;
                            let rs1 = parser.get_reg()?;
                            let raw_token = parser.take_raw_token()?;
                            Ok(RVInstructionNode::new_csr(
                                With::new(inst, next_node),
                                rd,
                                csr,
                                rs1,
                                raw_token,
                                parser.current_segment,
                                mem::take(&mut parser.current_labels),
                            ))
                        }
                        RVInstType::UpperArith(inst) => {
                            let rd = parser.get_reg()?;
                            let mut imm = parser.get_imm()?;
                            if let Some(value) = imm.get().value() {
                                let new_imm = Imm::new(value << 12);
                                // shift left by 12
                                *imm.get_mut() = new_imm;
                            }
                            let raw_token = parser.take_raw_token()?;
                            Ok(RVInstructionNode::new_iarith(
                                With::new(inst, next_node.clone()),
                                rd,
                                With::new(RVRegister::X0, next_node),
                                imm,
                                raw_token,
                                parser.current_segment,
                                mem::take(&mut parser.current_labels),
                            ))
                        }
                        RVInstType::Arith(inst) => {
                            let rd = parser.get_reg()?;
                            let rs1 = parser.get_reg()?;
                            let rs2 = parser.get_reg()?;
                            let raw_token = parser.take_raw_token()?;
                            Ok(RVInstructionNode::new_arith(
                                With::new(inst, next_node),
                                rd,
                                rs1,
                                rs2,
                                raw_token,
                                parser.current_segment,
                                mem::take(&mut parser.current_labels),
                            ))
                        }
                        RVInstType::IArith(inst) => {
                            let rd = parser.get_reg()?;
                            let rs1 = parser.get_reg()?;
                            let imm = parser.get_imm()?;
                            let raw_token = parser.take_raw_token()?;
                            Ok(RVInstructionNode::new_iarith(
                                With::new(inst, next_node),
                                rd,
                                rs1,
                                imm,
                                raw_token,
                                parser.current_segment,
                                mem::take(&mut parser.current_labels),
                            ))
                        }

                        RVInstType::JumpLink(inst) => {
                            if inst == JumpLinkType::Tail {
                                let name = parser.get_label()?;
                                let raw_token = parser.take_raw_token()?;
                                return Ok(RVInstructionNode::new_jump_link(
                                    With::new(inst, next_node.clone()),
                                    With::new(RVRegister::X0, next_node),
                                    name,
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ));
                            }

                            let next = parser.get_any()?;

                            return if let Ok(reg) = next.as_reg() {
                                let name = parser.get_label()?;
                                let raw_token = parser.take_raw_token()?;
                                Ok(RVInstructionNode::new_jump_link(
                                    With::new(inst, next_node),
                                    reg,
                                    name,
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ))
                            } else if let Ok(name) = next.as_label() {
                                let raw_token = parser.take_raw_token()?;
                                Ok(RVInstructionNode::new_jump_link(
                                    With::new(inst, next_node.clone()),
                                    With::new(RVRegister::X1, next_node),
                                    name,
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ))
                            } else {
                                Err(LexError::Expected(
                                    vec![ExpectedType::Register, ExpectedType::Label],
                                    Box::new(next),
                                ))
                            };
                        }
                        RVInstType::JumpLinkR(inst) => {
                            let reg1 = parser.get_reg()?;
                            let next = parser.get_any()?;
                            return if let Ok(rs1) = next.as_reg() {
                                let raw_token = parser.take_raw_token()?;
                                let imm = parser.get_imm()?;
                                Ok(RVInstructionNode::new_jump_link_r(
                                    With::new(inst, next_node),
                                    reg1,
                                    rs1,
                                    imm,
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ))
                            } else if let Ok(imm) = next.as_imm() {
                                if let Ok(()) = parser.peek_any()?.as_lparen() {
                                    parser.get_any()?;
                                    let rs1 = parser.get_reg()?;
                                    parser.expect_rparen()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_jump_link_r(
                                        With::new(inst, next_node),
                                        reg1,
                                        rs1,
                                        imm,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                } else {
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_jump_link_r(
                                        With::new(inst, next_node.clone()),
                                        With::new(RVRegister::X1, next_node),
                                        reg1,
                                        imm,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                            } else if let Ok(()) = next.as_lparen() {
                                let rs1 = parser.get_reg()?;
                                parser.expect_rparen()?;
                                let raw_token = parser.take_raw_token()?;
                                Ok(RVInstructionNode::new_jump_link_r(
                                    With::new(inst, next_node.clone()),
                                    reg1,
                                    rs1,
                                    With::new(Imm::new(0), next_node),
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ))
                            } else {
                                let raw_token = parser.take_raw_token()?;
                                Ok(RVInstructionNode::new_jump_link_r(
                                    With::new(inst, next_node.clone()),
                                    With::new(RVRegister::X1, next_node.clone()),
                                    reg1,
                                    With::new(Imm::new(0), next_node),
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ))
                            };
                        }
                        RVInstType::Load(inst) => {
                            let rd = parser.get_reg()?;
                            let next = parser.get_any()?;
                            return if let Ok(imm) = next.as_imm() {
                                if let Ok(()) = parser.peek_any()?.as_lparen() {
                                    parser.get_any()?;
                                    let rs1 = parser.get_reg()?;
                                    parser.expect_rparen()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_load(
                                        With::new(inst, next_node),
                                        rd,
                                        rs1,
                                        imm,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                } else {
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_load(
                                        With::new(inst, next_node.clone()),
                                        rd,
                                        With::new(RVRegister::X0, next_node),
                                        imm,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                            } else if let Ok(label) = next.as_label() {
                                let raw_token = parser.take_raw_token()?;
                                parser.queue.push_back(RVInstructionNode::new_load(
                                    With::new(inst, next_node.clone()),
                                    rd.clone(),
                                    rd.clone(),
                                    With::new(Imm::new(0), next_node.clone()),
                                    raw_token.clone(),
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ));
                                Ok(RVInstructionNode::new_load_addr(
                                    With::new(PseudoType::La, next_node.clone()),
                                    rd.clone(),
                                    label,
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ))
                            } else if let Ok(()) = next.as_lparen() {
                                let rs1 = parser.get_reg()?;
                                parser.expect_rparen()?;
                                let raw_token = parser.take_raw_token()?;
                                Ok(RVInstructionNode::new_load(
                                    With::new(inst, next_node.clone()),
                                    rd,
                                    rs1,
                                    With::new(Imm::new(0), next_node),
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ))
                            } else {
                                Err(LexError::Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    Box::new(next),
                                ))
                            };
                        }
                        RVInstType::Store(inst) => {
                            let rs2 = parser.get_reg()?;
                            let next = parser.get_any()?;

                            return if let Ok(imm) = next.as_imm() {
                                if let Ok(()) = parser.peek_any()?.as_lparen() {
                                    parser.get_any()?;
                                    let rs1 = parser.get_reg()?;
                                    parser.expect_rparen()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_store(
                                        With::new(inst, next_node),
                                        rs1,
                                        rs2,
                                        imm,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                } else if let Ok(tmp) = parser.peek_any()?.as_reg() {
                                    parser.get_any()?;
                                    let raw_token = parser.take_raw_token()?;
                                    parser.queue.push_back(RVInstructionNode::new_store(
                                        With::new(inst, next_node.clone()),
                                        tmp.clone(),
                                        rs2,
                                        With::new(Imm::new(0), next_node.clone()),
                                        raw_token.clone(),
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ));
                                    Ok(RVInstructionNode::new_iarith(
                                        With::new(IArithType::Addi, next_node.clone()),
                                        tmp.clone(),
                                        With::new(RVRegister::X0, next_node.clone()),
                                        imm,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                } else {
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_store(
                                        With::new(inst, next_node.clone()),
                                        With::new(RVRegister::X0, next_node),
                                        rs2,
                                        imm,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                            } else if let Ok(label) = next.as_label() {
                                let temp_reg = parser.get_reg()?;
                                let raw_token = parser.take_raw_token()?;
                                parser.queue.push_back(RVInstructionNode::new_store(
                                    With::new(inst, next_node.clone()),
                                    temp_reg.clone(),
                                    rs2.clone(),
                                    With::new(Imm::new(0), next_node.clone()),
                                    raw_token.clone(),
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ));
                                Ok(RVInstructionNode::new_load_addr(
                                    With::new(PseudoType::La, next_node.clone()),
                                    temp_reg.clone(),
                                    label,
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ))
                            } else if let Ok(()) = next.as_lparen() {
                                let rs1 = parser.get_reg()?;
                                parser.expect_rparen()?;
                                let raw_token = parser.take_raw_token()?;
                                Ok(RVInstructionNode::new_store(
                                    With::new(inst, next_node.clone()),
                                    rs1,
                                    rs2,
                                    With::new(Imm::new(0), next_node),
                                    raw_token,
                                    parser.current_segment,
                                    mem::take(&mut parser.current_labels),
                                ))
                            } else {
                                Err(LexError::Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    Box::new(next),
                                ))
                            };
                        }
                        RVInstType::Branch(inst) => {
                            let rs1 = parser.get_reg()?;
                            let rs2 = parser.get_reg()?;
                            let label = parser.get_label()?;
                            let raw_token = parser.take_raw_token()?;
                            Ok(RVInstructionNode::new_branch(
                                With::new(inst, next_node),
                                rs1,
                                rs2,
                                label,
                                raw_token,
                                parser.current_segment,
                                mem::take(&mut parser.current_labels),
                            ))
                        }
                        RVInstType::Ignore(_) => {
                            Err(LexError::IgnoredWithWarning(Box::new(next_node)))
                        }
                        RVInstType::Basic(inst) => Ok(RVInstructionNode::new_basic(
                            With::new(inst, next_node),
                            parser.take_raw_token()?,
                            parser.current_segment,
                            mem::take(&mut parser.current_labels),
                        )),
                        RVInstType::Pseudo(inst) => {
                            return match inst {
                                PseudoType::Ret => {
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_jump_link_r(
                                        With::new(JumpLinkRType::Jalr, next_node.clone()),
                                        With::new(RVRegister::X0, next_node.clone()),
                                        With::new(RVRegister::X1, next_node.clone()),
                                        With::new(Imm::new(0), next_node.clone()),
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Mv => {
                                    let rd = parser.get_reg()?;
                                    let rs1 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_arith(
                                        With::new(ArithType::Add, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(RVRegister::X0, next_node.clone()),
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Li => {
                                    let rd = parser.get_reg()?;
                                    let imm = parser.get_imm()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_iarith(
                                        With::new(IArithType::Addi, next_node.clone()),
                                        rd,
                                        With::new(RVRegister::X0, imm.token().clone()),
                                        imm,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::La => {
                                    let rd = parser.get_reg()?;
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_load_addr(
                                        With::new(PseudoType::La, next_node.clone()),
                                        rd,
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::J | PseudoType::B => {
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_jump_link(
                                        With::new(JumpLinkType::Jal, next_node.clone()),
                                        With::new(RVRegister::X0, next_node.clone()),
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Jr => {
                                    let rs1 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_jump_link_r(
                                        With::new(JumpLinkRType::Jalr, next_node.clone()),
                                        With::new(RVRegister::X0, next_node.clone()),
                                        rs1,
                                        With::new(Imm::new(0), next_node.clone()),
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Beqz => {
                                    let rs1 = parser.get_reg()?;
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_branch(
                                        With::new(BranchType::Beq, next_node.clone()),
                                        rs1,
                                        With::new(RVRegister::X0, next_node.clone()),
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Bnez => {
                                    let rs1 = parser.get_reg()?;
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_branch(
                                        With::new(BranchType::Bne, next_node.clone()),
                                        rs1,
                                        With::new(RVRegister::X0, next_node.clone()),
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Bltz | PseudoType::Bgtz => {
                                    let rs1 = parser.get_reg()?;
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_branch(
                                        With::new(BranchType::Blt, next_node.clone()),
                                        rs1,
                                        With::new(RVRegister::X0, next_node.clone()),
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Neg => {
                                    let rd = parser.get_reg()?;
                                    let rs1 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_arith(
                                        With::new(ArithType::Sub, next_node.clone()),
                                        rd,
                                        With::new(RVRegister::X0, next_node.clone()),
                                        rs1,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Not => {
                                    let rd = parser.get_reg()?;
                                    let rs1 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_iarith(
                                        With::new(IArithType::Xori, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(Imm::new(-1), next_node.clone()),
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Seqz => {
                                    let rd = parser.get_reg()?;
                                    let rs1 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_iarith(
                                        With::new(IArithType::Sltiu, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(Imm::new(1), next_node.clone()),
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Snez => {
                                    let rd = parser.get_reg()?;
                                    let rs1 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_iarith(
                                        With::new(IArithType::Sltiu, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(Imm::new(0), next_node.clone()),
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Nop => {
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_iarith(
                                        With::new(IArithType::Addi, next_node.clone()),
                                        With::new(RVRegister::X0, next_node.clone()),
                                        With::new(RVRegister::X0, next_node.clone()),
                                        With::new(Imm::new(0), next_node.clone()),
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Bgez | PseudoType::Blez => {
                                    let rs1 = parser.get_reg()?;
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_branch(
                                        With::new(BranchType::Bge, next_node.clone()),
                                        rs1,
                                        With::new(RVRegister::X0, next_node.clone()),
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Sgtz => {
                                    let rd = parser.get_reg()?;
                                    let rs1 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_arith(
                                        With::new(ArithType::Slt, next_node.clone()),
                                        rd,
                                        With::new(RVRegister::X0, next_node.clone()),
                                        rs1,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Sltz => {
                                    let rd = parser.get_reg()?;
                                    let rs1 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_arith(
                                        With::new(ArithType::Slt, next_node.clone()),
                                        rd,
                                        rs1,
                                        With::new(RVRegister::X0, next_node.clone()),
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Sgt => {
                                    let rd = parser.get_reg()?;
                                    let rs1 = parser.get_reg()?;
                                    let rs2 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_arith(
                                        With::new(ArithType::Slt, next_node.clone()),
                                        rd,
                                        rs2,
                                        rs1,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Sgtu => {
                                    let rd = parser.get_reg()?;
                                    let rs1 = parser.get_reg()?;
                                    let rs2 = parser.get_reg()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_arith(
                                        With::new(ArithType::Sltu, next_node.clone()),
                                        rd,
                                        rs2,
                                        rs1,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Call => {
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_jump_link(
                                        With::new(JumpLinkType::Jal, next_node.clone()),
                                        With::new(RVRegister::X1, next_node.clone()),
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Bgt => {
                                    let rs1 = parser.get_reg()?;
                                    let rs2 = parser.get_reg()?;
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_branch(
                                        With::new(BranchType::Blt, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Ble => {
                                    let rs1 = parser.get_reg()?;
                                    let rs2 = parser.get_reg()?;
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_branch(
                                        With::new(BranchType::Bge, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Bgtu => {
                                    let rs1 = parser.get_reg()?;
                                    let rs2 = parser.get_reg()?;
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_branch(
                                        With::new(BranchType::Bltu, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Bleu => {
                                    let rs1 = parser.get_reg()?;
                                    let rs2 = parser.get_reg()?;
                                    let label = parser.get_label()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_branch(
                                        With::new(BranchType::Bgeu, next_node.clone()),
                                        rs2,
                                        rs1,
                                        label,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Csrci | PseudoType::Csrsi | PseudoType::Csrwi => {
                                    let csr = parser.get_csrimm()?;
                                    let imm = parser.get_imm()?;
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
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_csri(
                                        With::new(inst, next_node.clone()),
                                        With::new(RVRegister::X0, next_node.clone()),
                                        csr,
                                        imm,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Csrc | PseudoType::Csrs | PseudoType::Csrw => {
                                    let rs1 = parser.get_reg()?;
                                    let csr = parser.get_csrimm()?;
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
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_csr(
                                        With::new(inst, next_node.clone()),
                                        With::new(RVRegister::X0, next_node.clone()),
                                        csr,
                                        rs1,
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                                PseudoType::Csrr => {
                                    let rd = parser.get_reg()?;
                                    let csr = parser.get_csrimm()?;
                                    let raw_token = parser.take_raw_token()?;
                                    Ok(RVInstructionNode::new_csr(
                                        With::new(CsrType::Csrrs, next_node.clone()),
                                        rd,
                                        csr,
                                        With::new(RVRegister::X0, next_node.clone()),
                                        raw_token,
                                        parser.current_segment,
                                        mem::take(&mut parser.current_labels),
                                    ))
                                }
                            }
                        }
                    };
                    return node;
                }

                if let Ok(directive) = DirectiveToken::from_str(s) {
                    return match directive {
                        DirectiveToken::Align => {
                            let _ = parser.get_imm()?;
                            Err(LexError::IgnoredWithoutWarning)
                        }
                        DirectiveToken::Ascii | DirectiveToken::Asciz | DirectiveToken::String => {
                            let _ = parser.get_string()?;
                            Err(LexError::IgnoredWithoutWarning)
                        }
                        DirectiveToken::Byte
                        | DirectiveToken::Double
                        | DirectiveToken::Dword
                        | DirectiveToken::Float
                        | DirectiveToken::Word
                        | DirectiveToken::Half => {
                            // keep looping through values until immediate or nl is
                            // not found
                            let mut values = Vec::new();
                            loop {
                                let next = parser.peek_any()?;
                                if let TokenType::Newline = next.token_type() {
                                    // consume newline
                                    parser.get_any()?;
                                } else if let Ok(imm) = next.as_imm() {
                                    // try to get immediate
                                    parser.get_any()?;
                                    values.push(imm);
                                } else {
                                    break;
                                }
                            }

                            Err(LexError::IgnoredWithoutWarning)
                        }
                        DirectiveToken::Data => {
                            parser.current_segment = Segment::Data;
                            Err(LexError::IgnoredWithoutWarning)
                        }
                        DirectiveToken::Macro => {
                            // macros are unsupported
                            // we will just ignore them until we reach endmacro
                            loop {
                                let next = parser.get_any()?;
                                if let TokenType::Symbol(dir2) = next.token_type() {
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
                        DirectiveToken::Section => {
                            // We have jumped to an unknown section.
                            // For our purposes, we will skip until we reach a directive token
                            // we care about.
                            parser.current_segment = Segment::Unknown;
                            Err(LexError::IgnoredWithoutWarning)
                        }
                        DirectiveToken::Extern | DirectiveToken::Eqv => {
                            Err(LexError::UnsupportedDirective(Box::new(next_node)))
                        }
                        DirectiveToken::Global | DirectiveToken::Globl => {
                            let label = parser.get_label()?;
                            Err(LexError::GlobalDef(Box::new(label)))
                        }
                        DirectiveToken::Include => {
                            let filename = parser.get_string()?;
                            Err(LexError::IncludeFile(Box::new(filename)))
                        }
                        DirectiveToken::Space => {
                            let _ = parser.get_imm()?;
                            Err(LexError::IgnoredWithoutWarning)
                        }
                        DirectiveToken::Text => {
                            parser.current_segment = Segment::Text;
                            Err(LexError::IgnoredWithoutWarning)
                        }
                        DirectiveToken::Type => {
                            let label = parser.get_label()?;
                            let symbol_type = parser.get_symbol()?;
                            if symbol_type.get() == "@function" {
                                Err(LexError::GlobalDef(Box::new(label)))
                            } else {
                                Err(LexError::IgnoredWithoutWarning)
                            }
                        }
                    };
                }

                // Unknown symbol; if it begins with a period, assume it is a directive
                if s.starts_with('.') {
                    Err(LexError::UnknownDirective(Box::new(next_node.clone())))
                } else {
                    Err(LexError::Expected(
                        vec![ExpectedType::Inst, ExpectedType::Directive],
                        Box::new(next_node.clone()),
                    ))
                }
            }
            TokenType::Label(s) => {
                let label = With::new(LabelString::new(s), next_node);
                parser.current_labels.insert(label.clone());
                parser.all_defined_labels.insert(label);
                Err(LexError::IgnoredWithoutWarning)
            }
            TokenType::Newline => Err(LexError::IsNewline(Box::new(next_node))),
            TokenType::LParen
            | TokenType::RParen
            | TokenType::String(_)
            | TokenType::Char(_)
            | TokenType::PercentHigh
            | TokenType::PercentLow
            | TokenType::Plus(_) => Err(LexError::UnexpectedToken(Box::new(next_node))),
            // Skip comment token
            TokenType::Comment(_) => Err(LexError::IgnoredWithoutWarning),
        }
    }
}
