#![allow(dead_code)]

use std::rc::Rc;
use uuid::Uuid;

use crate::cfg::BasicBlock;
use crate::parser::ASTNode;
use crate::parser::Lexer;
use crate::parser::{Position, Range, Token, TokenInfo, With};

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

impl<T> With<T>
where
    T: PartialEq<T>,
{
    // TODO should only be used in testing, get rid of later
    pub fn blank(data: T) -> Self {
        With {
            token: Token::Symbol(String::new()),
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
            With::blank(ArithType::$inst),
            With::blank(Register::$rd),
            With::blank(Register::$rs1),
            With::blank(Register::$rs2),
        )
    };
}

#[macro_export]
macro_rules! iarith {
    ($inst:ident $rd:ident $rs1:ident $imm:expr) => {
        ASTNode::new_iarith(
            With::blank(IArithType::$inst),
            With::blank(Register::$rd),
            With::blank(Register::$rs1),
            With::blank(Imm($imm)),
        )
    };
}

#[macro_export]
macro_rules! load {
    ($inst:ident $rd:ident $rs1:ident $imm:expr ) => {
        ASTNode::new_load(
            With::blank(LoadType::$inst),
            With::blank(Register::$rd),
            With::blank(Register::$rs1),
            With::blank(Imm($imm)),
        )
    };
}

#[macro_export]
macro_rules! store {
    ($inst:ident $rd:ident $rs1:ident $imm:expr ) => {
        ASTNode::new_store(
            With::blank(StoreType::$inst),
            With::blank(Register::$rd),
            With::blank(Register::$rs1),
            With::blank(Imm($imm)),
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
            offset: With::blank(Imm($a)),
            reg: With::blank(Register::$b),
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
