use uuid::Uuid;

use crate::cfg::Cfg;
use crate::lsp::CanGetURIString;
use crate::parser::inst::{
    ArithType, BranchType, CSRIType, CSRType, IArithType, Inst, JumpLinkRType, JumpLinkType,
    PseudoType, Type,
};
use crate::parser::token::With;
use crate::parser::{DataType, RawToken, Register};
use crate::parser::{DirectiveToken, LexError};
use crate::parser::{DirectiveType, ParserNode};
use crate::parser::{Lexer, Token};
use crate::passes::{DiagnosticItem, Manager};
use crate::reader::FileReader;
use std::collections::HashSet;
use std::iter::Peekable;
use std::str::FromStr;

use super::imm::{CSRImm, Imm};
use super::token::Info;
use super::{ExpectedType, LabelString, ParseError, Range};

/// Parser for RISC-V assembly
pub struct RVParser<T>
where
    T: FileReader + Clone,
{
    lexer_stack: Vec<Peekable<Lexer>>,
    pub reader: T,
}
impl<T> RVParser<T>
where
    T: CanGetURIString + Clone + FileReader,
{
    pub fn get_full_url(&mut self, path: &str, uuid: Uuid) -> String {
        let doc = self.reader.get_uri_string(uuid);
        let uri = lsp_types::Url::parse(&doc.uri).unwrap();
        let fileuri = uri.join(path).unwrap();
        fileuri.to_string()
    }
}
impl<T: FileReader + Clone + CanGetURIString> RVParser<T> {
    /// Return the imported files of a file
    pub fn get_imports(&mut self, base: &str) -> HashSet<String> {
        let mut imported = HashSet::new();
        let items = self.parse(base, true);
        for item in items.0 {
            match item {
                ParserNode::Directive(x) => match x.dir {
                    DirectiveType::Include(name) => {
                        // get full file path
                        let this_uri = self.get_full_url(&name.data, x.dir_token.file);
                        // add to set
                        imported.insert(this_uri);
                        // imports.insert(this_uri);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        imported
    }
}
impl<T: FileReader + Clone> RVParser<T> {
    pub fn run(&mut self, base: &str, debug: bool) -> Vec<DiagnosticItem> {
        let mut diags = Vec::new();
        let parsed = self.parse(base, false);
        parsed
            .1
            .iter()
            .for_each(|x| diags.push(DiagnosticItem::from(x.clone())));

        let cfg = match Cfg::new(parsed.0) {
            Ok(cfg) => cfg,
            Err(err) => {
                diags.push(DiagnosticItem::from(*err));
                diags.sort();
                return diags;
            }
        };

        let res = Manager::run(cfg.clone(), debug);
        match res {
            Ok(lints) => {
                lints
                    .iter()
                    .for_each(|x| diags.push(DiagnosticItem::from(x.clone())));
            }
            Err(err) => diags.push(DiagnosticItem::from(*err)),
        }
        diags.sort();
        diags
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
            Err(e) => {
                parse_errors.push(e.to_parse_error(With::new(base.to_owned(), Info::default())));
                return (nodes, parse_errors);
            }
        };
        self.lexer_stack.push(lexer.1);

        // Add program entry node
        nodes.push(ParserNode::new_program_entry(
            lexer.0,
            RawToken {
                text: "".to_string(),
                pos: Range::default(),
                file: lexer.0,
            },
        ));

        while let Some(lexer) = self.lexer() {
            let node = ParserNode::try_from(lexer);

            match node {
                Ok(x) => {
                    if !ignore_imports {
                        if let ParserNode::Directive(directive) = &x {
                            if let DirectiveType::Include(path) = &directive.dir {
                                let lexer = self.reader.import_file(
                                    path.data.as_str(),
                                    Some(directive.dir_token.file),
                                );
                                match lexer {
                                    Ok(x) => {
                                        self.lexer_stack.push(x.1);
                                    }
                                    Err(x) => {
                                        parse_errors.push(x.to_parse_error(path.clone()));
                                    }
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
                        self.recover_from_parse_error();
                    }
                    LexError::UnknownDirective(y) => {
                        parse_errors.push(ParseError::UnknownDirective(y));
                        self.recover_from_parse_error();
                    }
                    LexError::UnsupportedDirective(y) => {
                        parse_errors.push(ParseError::Unsupported(y));
                        self.recover_from_parse_error();
                    }
                },
            }
        }
        (nodes, parse_errors)
    }

    fn lexer(&mut self) -> Option<&mut Peekable<Lexer>> {
        let item = &mut self.lexer_stack;
        item.last_mut()
    }
}

impl Info {
    fn as_lparen(&self) -> Result<(), LexError> {
        match self.token {
            Token::LParen => Ok(()),
            _ => Err(LexError::Expected(vec![ExpectedType::LParen], self.clone())),
        }
    }

    fn as_rparen(&self) -> Result<(), LexError> {
        match self.token {
            Token::RParen => Ok(()),
            _ => Err(LexError::Expected(vec![ExpectedType::RParen], self.clone())),
        }
    }

    fn as_reg(&self) -> Result<With<Register>, LexError> {
        With::<Register>::try_from(self.clone())
            .map_err(|_| LexError::Expected(vec![ExpectedType::Register], self.clone()))
    }

    fn as_imm(&self) -> Result<With<Imm>, LexError> {
        With::<Imm>::try_from(self.clone())
            .map_err(|_| LexError::Expected(vec![ExpectedType::Imm], self.clone()))
    }

    fn as_label(&self) -> Result<With<LabelString>, LexError> {
        With::<LabelString>::try_from(self.clone())
            .map_err(|_| LexError::Expected(vec![ExpectedType::Label], self.clone()))
    }

    fn as_csrimm(&self) -> Result<With<CSRImm>, LexError> {
        With::<CSRImm>::try_from(self.clone())
            .map_err(|_| LexError::Expected(vec![ExpectedType::CSRImm], self.clone()))
    }

    fn as_string(&self) -> Result<With<String>, LexError> {
        With::<String>::try_from(self.clone())
            .map_err(|_| LexError::Expected(vec![ExpectedType::String], self.clone()))
    }
}

impl<'a> AnnotatedLexer<'a> {
    fn _expect_lparen(&mut self) -> Result<(), LexError> {
        self.get_any()?.as_lparen()
    }

    fn expect_rparen(&mut self) -> Result<(), LexError> {
        self.get_any()?.as_rparen()
    }

    fn get_reg(&mut self) -> Result<With<Register>, LexError> {
        self.get_any()?.as_reg()
    }

    fn get_imm(&mut self) -> Result<With<Imm>, LexError> {
        self.get_any()?.as_imm()
    }

    fn get_label(&mut self) -> Result<With<LabelString>, LexError> {
        self.get_any()?.as_label()
    }

    fn get_csrimm(&mut self) -> Result<With<CSRImm>, LexError> {
        self.get_any()?.as_csrimm()
    }

    fn get_string(&mut self) -> Result<With<String>, LexError> {
        self.get_any()?.as_string()
    }

    fn get_any(&mut self) -> Result<Info, LexError> {
        let item = self.lexer.next().ok_or(LexError::UnexpectedEOF)?;
        if self.raw_token == RawToken::default() {
            self.raw_token = RawToken {
                text: item.token.as_original_string(),
                pos: item.pos.clone(),
                file: item.file,
            };
        } else {
            self.raw_token.text.push_str(" ");
            self.raw_token
                .text
                .push_str(&item.token.as_original_string());
            self.raw_token.pos.end = item.pos.end;
        }
        Ok(item)
    }

    fn peek_any(&mut self) -> Result<&Info, LexError> {
        self.lexer.peek().ok_or(LexError::UnexpectedEOF)
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
        use LexError::{Expected, Ignored, IsNewline, NeedTwoNodes};

        let mut lex = AnnotatedLexer {
            lexer: val,
            raw_token: RawToken::default(),
        };

        let next_node = lex.get_any()?;
        match &next_node.token {
            Token::Symbol(s) => {
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
                            // shift left by 12
                            imm.data.0 <<= 12;
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
                                    next,
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
                                    With::new(Imm(0), next_node),
                                    lex.raw_token,
                                ))
                            } else {
                                Ok(ParserNode::new_jump_link_r(
                                    With::new(inst, next_node.clone()),
                                    With::new(Register::X1, next_node.clone()),
                                    reg1,
                                    With::new(Imm(0), next_node),
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
                                        With::new(Imm(0), next_node),
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
                                    With::new(Imm(0), next_node),
                                    lex.raw_token,
                                ))
                            } else {
                                Err(Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    next,
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
                                            tmp.clone(),
                                            rs2,
                                            With::new(Imm(0), next_node),
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
                                        With::new(Imm(0), next_node),
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
                                    With::new(Imm(0), next_node),
                                    lex.raw_token,
                                ))
                            } else {
                                Err(Expected(
                                    vec![
                                        ExpectedType::Label,
                                        ExpectedType::Imm,
                                        ExpectedType::LParen,
                                    ],
                                    next,
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
                        Type::Ignore(_) => Err(Ignored(next_node)),
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
                                    With::new(Imm(0), next_node.clone()),
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
                                    With::new(Register::X0, imm.info()),
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
                                    With::new(Imm(0), next_node.clone()),
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
                                    With::new(Imm(-1), next_node.clone()),
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
                                    With::new(Imm(1), next_node.clone()),
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
                                    With::new(Imm(0), next_node.clone()),
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Nop => {
                                return Ok(ParserNode::new_iarith(
                                    With::new(IArithType::Addi, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    With::new(Register::X0, next_node.clone()),
                                    With::new(Imm(0), next_node.clone()),
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
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Csrc | PseudoType::Csrs | PseudoType::Csrw => {
                                let rs1 = lex.get_reg()?;
                                let csr = lex.get_csrimm()?;
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
                                    lex.raw_token,
                                ));
                            }
                            PseudoType::Csrr => {
                                let rd = lex.get_reg()?;
                                let csr = lex.get_csrimm()?;
                                return Ok(ParserNode::new_csr(
                                    With::new(CSRType::Csrrs, next_node.clone()),
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
                    next_node.clone(),
                ))
            }
            Token::Label(s) => Ok(ParserNode::new_label(
                With::new(
                    LabelString::from_str(s).map_err(|_| {
                        LexError::Expected(vec![ExpectedType::Label], next_node.clone())
                    })?,
                    next_node,
                ),
                lex.raw_token,
            )),
            Token::Directive(dir) => {
                if let Ok(directive) = DirectiveToken::from_str(dir) {
                    match directive {
                        DirectiveToken::Align => {
                            let imm = lex.get_imm()?;
                            return Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Align(imm),
                                lex.raw_token,
                            ));
                        }
                        DirectiveToken::Ascii => {
                            let string = lex.get_string()?;
                            return Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Ascii {
                                    text: string.clone(),
                                    null_term: false,
                                },
                                lex.raw_token,
                            ));
                        }
                        DirectiveToken::Asciz | DirectiveToken::String => {
                            let string = lex.get_string()?;
                            return Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Ascii {
                                    text: string.clone(),
                                    null_term: true,
                                },
                                lex.raw_token,
                            ));
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
                                _ => unreachable!(),
                            };

                            // keep looping through values until immediate or nl is
                            // not found
                            let mut values = Vec::new();
                            loop {
                                let next = lex.peek_any()?;
                                if let Token::Newline = next.token {
                                    // consume newline
                                    lex.get_any()?;
                                    continue;
                                } else if let Ok(imm) = next.as_imm() {
                                    // try to get immediate
                                    lex.get_any()?;
                                    values.push(imm);
                                } else {
                                    break;
                                }
                            }

                            return Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Data(data_type, values),
                                lex.raw_token,
                            ));
                        }
                        DirectiveToken::Data => {
                            return Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::DataSection,
                                lex.raw_token,
                            ));
                        }
                        DirectiveToken::Macro => {
                            // macros are unsupported
                            // we will just ignore them until the we reach endmacro
                            loop {
                                let next = lex.get_any()?;
                                if let Token::Directive(dir) = next.token {
                                    if let Ok(directive) = DirectiveToken::from_str(&dir) {
                                        if directive == DirectiveToken::EndMacro {
                                            break;
                                        }
                                    }
                                }
                            }
                            return Err(LexError::Ignored(next_node));
                        }
                        DirectiveToken::EndMacro => return Err(LexError::Ignored(next_node)),
                        DirectiveToken::Eqv => {
                            return Err(LexError::UnsupportedDirective(next_node));
                        }
                        DirectiveToken::Global | DirectiveToken::Globl => {
                            return Err(LexError::UnsupportedDirective(next_node));
                        }
                        DirectiveToken::Include => {
                            let filename = lex.get_string()?;
                            return Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Include(filename),
                                lex.raw_token,
                            ));
                        }
                        DirectiveToken::Section => {
                            return Err(LexError::UnsupportedDirective(next_node));
                        }
                        DirectiveToken::Extern => {
                            return Err(LexError::UnsupportedDirective(next_node));
                        }
                        DirectiveToken::Space => {
                            let imm = lex.get_imm()?;
                            return Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::Space(imm),
                                lex.raw_token,
                            ));
                        }
                        DirectiveToken::Text => {
                            return Ok(ParserNode::new_directive(
                                With::new(directive, next_node.clone()),
                                DirectiveType::TextSection,
                                lex.raw_token,
                            ));
                        }
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
