use crate::parser::token::{Token, TokenInfo};
use std::{convert::TryFrom, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    X0,
    X1,
    X2,
    X3,
    X4,
    X5,
    X6,
    X7,
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    X26,
    X27,
    X28,
    X29,
    X30,
    X31,
}

impl TryFrom<TokenInfo> for Register {
    type Error = ();

    fn try_from(value: TokenInfo) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => Register::from_str(&s),
            _ => Err(()),
        }
    }
}

impl FromStr for Register {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "x0" | "zero" => Ok(Register::X0),
            "x1" | "ra" => Ok(Register::X1),
            "x2" | "sp" => Ok(Register::X2),
            "x3" | "gp" => Ok(Register::X3),
            "x4" | "tp" => Ok(Register::X4),
            "x5" | "t0" => Ok(Register::X5),
            "x6" | "t1" => Ok(Register::X6),
            "x7" | "t2" => Ok(Register::X7),
            "x8" | "s0" | "fp" => Ok(Register::X8),
            "x9" | "s1" => Ok(Register::X9),
            "x10" | "a0" => Ok(Register::X10),
            "x11" | "a1" => Ok(Register::X11),
            "x12" | "a2" => Ok(Register::X12),
            "x13" | "a3" => Ok(Register::X13),
            "x14" | "a4" => Ok(Register::X14),
            "x15" | "a5" => Ok(Register::X15),
            "x16" | "a6" => Ok(Register::X16),
            "x17" | "a7" => Ok(Register::X17),
            "x18" | "s2" => Ok(Register::X18),
            "x19" | "s3" => Ok(Register::X19),
            "x20" | "s4" => Ok(Register::X20),
            "x21" | "s5" => Ok(Register::X21),
            "x22" | "s6" => Ok(Register::X22),
            "x23" | "s7" => Ok(Register::X23),
            "x24" | "s8" => Ok(Register::X24),
            "x25" | "s9" => Ok(Register::X25),
            "x26" | "s10" => Ok(Register::X26),
            "x27" | "s11" => Ok(Register::X27),
            "x28" | "t3" => Ok(Register::X28),
            "x29" | "t4" => Ok(Register::X29),
            "x30" | "t5" => Ok(Register::X30),
            "x31" | "t6" => Ok(Register::X31),
            _ => Err(()),
        }
    }
}

impl ToString for Register {
    fn to_string(&self) -> String {
        match self {
            Register::X0 => "x0".to_owned(),
            Register::X1 => "x1".to_owned(),
            Register::X2 => "x2".to_owned(),
            Register::X3 => "x3".to_owned(),
            Register::X4 => "x4".to_owned(),
            Register::X5 => "x5".to_owned(),
            Register::X6 => "x6".to_owned(),
            Register::X7 => "x7".to_owned(),
            Register::X8 => "x8".to_owned(),
            Register::X9 => "x9".to_owned(),
            Register::X10 => "x10".to_owned(),
            Register::X11 => "x11".to_owned(),
            Register::X12 => "x12".to_owned(),
            Register::X13 => "x13".to_owned(),
            Register::X14 => "x14".to_owned(),
            Register::X15 => "x15".to_owned(),
            Register::X16 => "x16".to_owned(),
            Register::X17 => "x17".to_owned(),
            Register::X18 => "x18".to_owned(),
            Register::X19 => "x19".to_owned(),
            Register::X20 => "x20".to_owned(),
            Register::X21 => "x21".to_owned(),
            Register::X22 => "x22".to_owned(),
            Register::X23 => "x23".to_owned(),
            Register::X24 => "x24".to_owned(),
            Register::X25 => "x25".to_owned(),
            Register::X26 => "x26".to_owned(),
            Register::X27 => "x27".to_owned(),
            Register::X28 => "x28".to_owned(),
            Register::X29 => "x29".to_owned(),
            Register::X30 => "x30".to_owned(),
            Register::X31 => "x31".to_owned(),
        }
    }
}
