#![allow(dead_code)]

use std::rc::Rc;
use uuid::Uuid;

use crate::cfg::BasicBlock;
use crate::parser::ast::ASTNode;
use crate::parser::lexer::Lexer;
use crate::parser::token::{Position, Range, Token, TokenInfo, WithToken};

pub fn basic_block_from_nodes(nodes: Vec<ASTNode>) -> Rc<BasicBlock> {
    let mut rc_nodes = Vec::new();
    for node in nodes {
        rc_nodes.push(Rc::new(node));
    }
    Rc::new(BasicBlock(rc_nodes, Uuid::new_v4()))
}

pub fn tokenize<S: Into<String>>(input: S) -> Vec<TokenInfo> {
    Lexer::new(input).collect()
}

impl<T> WithToken<T>
where
    T: PartialEq<T>,
{
    // TODO should only be used in testing, get rid of later
    pub fn blank(data: T) -> Self {
        WithToken {
            token: Token::Symbol("".to_owned()),
            pos: Range {
                start: Position { line: 0, column: 0 },
                end: Position { line: 0, column: 0 },
            },
            data,
        }
    }
}

// to make prototyping easier, use the macro to create AST nodes
// example macro usage rtype!(Add X0 X1 X2)
#[macro_export]
macro_rules! arith {
    ($inst:ident $rd:ident $rs1:ident $rs2:ident) => {
        ASTNode::new_arith(
            WithToken::blank(ArithType::$inst),
            WithToken::blank(Register::$rd),
            WithToken::blank(Register::$rs1),
            WithToken::blank(Register::$rs2),
        )
    };
}

#[macro_export]
macro_rules! iarith {
    ($inst:ident $rd:ident $rs1:ident $imm:expr) => {
        ASTNode::new_iarith(
            WithToken::blank(IArithType::$inst),
            WithToken::blank(Register::$rd),
            WithToken::blank(Register::$rs1),
            WithToken::blank(Imm($imm)),
        )
    };
}

#[macro_export]
macro_rules! load {
    ($inst:ident $rd:ident $rs1:ident $imm:expr ) => {
        ASTNode::new_load(
            WithToken::blank(LoadType::$inst),
            WithToken::blank(Register::$rd),
            WithToken::blank(Register::$rs1),
            WithToken::blank(Imm($imm)),
        )
    };
}

#[macro_export]
macro_rules! store {
    ($inst:ident $rd:ident $rs1:ident $imm:expr ) => {
        ASTNode::new_store(
            WithToken::blank(StoreType::$inst),
            WithToken::blank(Register::$rd),
            WithToken::blank(Register::$rs1),
            WithToken::blank(Imm($imm)),
        )
    };
}

#[macro_export]
macro_rules! act {
    ($x:expr) => {
        Mem::try_from(token!($x)).unwrap()
    };
}

#[macro_export]
macro_rules! exp {
    ($a:expr, $b:ident) => {
        Mem {
            offset: WithToken::blank(Imm($a)),
            reg: WithToken::blank(Register::$b),
        }
    };
}

#[macro_export]
macro_rules! token {
    ($x:expr) => {
        TokenInfo {
            token: Token::Symbol($x.to_owned()),
            pos: Range {
                start: Position { line: 0, column: 0 },
                end: Position {
                    line: 0,
                    column: $x.len(),
                },
            },
        }
    };
}
