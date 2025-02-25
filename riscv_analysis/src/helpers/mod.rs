use crate::parser::{Range, RawToken, Token, With};

impl RawToken {
    #[must_use]
    pub fn blank() -> Self {
        RawToken {
            text: String::new(),
            pos: Range::default(),
            file: uuid::Uuid::nil(),
        }
    }
}

impl<T> With<T>
where
    T: PartialEq<T>,
{
    pub fn blank(data: T) -> Self {
        With {
            token: Token::Symbol(String::new()),
            pos: Range::default(),
            file: uuid::Uuid::nil(),
            data,
        }
    }
}

// to make prototyping easier, use the macro to create parser nodes
// example macro usage rtype!(Add X0 X1 X2)
#[macro_export]
macro_rules! arith {
    ($inst:ident $rd:ident $rs1:ident $rs2:ident) => {
        $crate::parser::ParserNode::new_arith(
            $crate::parser::With::blank($crate::parser::ArithType::$inst),
            $crate::parser::With::blank($crate::parser::Register::$rd),
            $crate::parser::With::blank($crate::parser::Register::$rs1),
            $crate::parser::With::blank($crate::parser::Register::$rs2),
            $crate::parser::RawToken::blank(),
        )
    };
}

#[macro_export]
macro_rules! iarith {
    ($inst:ident $rd:ident $rs1:ident $imm:expr) => {
        $crate::parser::ParserNode::new_iarith(
            $crate::parser::With::blank($crate::parser::IArithType::$inst),
            $crate::parser::With::blank($crate::parser::Register::$rd),
            $crate::parser::With::blank($crate::parser::Register::$rs1),
            $crate::parser::With::blank($crate::parser::Imm($imm)),
            $crate::parser::RawToken::blank(),
        )
    };
}

#[macro_export]
macro_rules! directive {
    ($dir_token:ident, $dir_type:ident) => {
        $crate::parser::ParserNode::new_directive(
            $crate::parser::With::blank($crate::parser::DirectiveToken::$dir_token),
            $crate::parser::DirectiveType::$dir_type,
            $crate::parser::RawToken::blank(),
        )
    };
}

#[macro_export]
macro_rules! load {
    ($inst:ident $rd:ident $rs1:ident $imm:expr ) => {
        $crate::parser::ParserNode::new_load(
            $crate::parser::With::blank($crate::parser::LoadType::$inst),
            $crate::parser::With::blank($crate::parser::Register::$rd),
            $crate::parser::With::blank($crate::parser::Register::$rs1),
            $crate::parser::With::blank($crate::parser::Imm($imm)),
            $crate::parser::RawToken::blank(),
        )
    };
}

#[macro_export]
macro_rules! store {
    ($inst:ident $rd:ident $rs1:ident $imm:expr ) => {
        $crate::parser::ParserNode::new_store(
            $crate::parser::With::blank($crate::parser::StoreType::$inst),
            $crate::parser::With::blank($crate::parser::Register::$rd),
            $crate::parser::With::blank($crate::parser::Register::$rs1),
            $crate::parser::With::blank($crate::parser::Imm($imm)),
            $crate::parser::RawToken::blank(),
        )
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
