#![allow(dead_code)]

use crate::parser::Lexer;
use crate::parser::{Info, Position, Range, Token, With};

pub fn tokenize<S: Into<String>>(input: S) -> Vec<Info> {
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
        ParserNode::new_arith(
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
        ParserNode::new_iarith(
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
        ParserNode::new_load(
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
        ParserNode::new_store(
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
        Info {
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
