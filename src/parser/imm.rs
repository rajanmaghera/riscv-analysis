use std::str::FromStr;

use crate::parser::token::{TokenInfo, Token, SymbolData};

#[derive(Debug, PartialEq, Clone)]
pub struct Imm(pub i32);

impl TryFrom<TokenInfo> for Imm {
    type Error = ();

    fn try_from(value: TokenInfo) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => Imm::try_from(s),
            _ => Err(()),
        }
    }
}

impl FromStr for Imm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") {
            match i32::from_str_radix(&s[2..], 16) {
                Ok(i) => return Ok(Imm(i)),
                Err(_) => return Err(()),
            }
        } else if s.starts_with("0b") {
            match i32::from_str_radix(&s[2..], 2) {
                Ok(i) => return Ok(Imm(i)),
                Err(_) => return Err(()),
            }
        } else {
            match s.parse::<i32>() {
                Ok(i) => Ok(Imm(i)),
                Err(_) => Err(()),
            }
        }
    }

}

impl TryFrom<SymbolData> for Imm {
    type Error = ();

    fn try_from(value: SymbolData) -> Result<Self, Self::Error> {
        Imm::from_str(&value.0)
    }
}

