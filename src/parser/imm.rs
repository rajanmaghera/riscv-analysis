use std::str::FromStr;

use crate::parser::token::{SymbolData, Token, TokenInfo};

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
        let neg = s.starts_with('-');
        let s = if neg { &s[1..] } else { s };
        let mul = if neg { -1 } else { 1 };

        if s.starts_with("0x") {
            if s[2..].starts_with('-') {
                return Err(());
            }
            match i32::from_str_radix(&s[2..], 16) {
                Ok(i) => return Ok(Imm(mul * i)),
                Err(_) => return Err(()),
            }
        } else if s.starts_with("0b") {
            if s[2..].starts_with('-') {
                return Err(());
            }
            match i32::from_str_radix(&s[2..], 2) {
                Ok(i) => return Ok(Imm(mul * i)),
                Err(_) => return Err(()),
            }
        } else {
            if s.starts_with('-') {
                return Err(());
            }
            match s.parse::<i32>() {
                Ok(i) => Ok(Imm(mul * i)),
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

#[cfg(test)]
mod test {
    use crate::parser::imm::Imm;
    use std::str::FromStr;

    #[test]
    fn basic_imm() {
        assert_eq!(Imm::from_str("0"), Ok(Imm(0)));
        assert_eq!(Imm::from_str("1"), Ok(Imm(1)));
        assert_eq!(Imm::from_str("-1"), Ok(Imm(-1)));
        assert_eq!(Imm::from_str("-16"), Ok(Imm(-16)));
    }

    #[test]
    fn no_spaces_allowed() {
        assert_eq!(Imm::from_str(" 0"), Err(()));
        assert_eq!(Imm::from_str("0 "), Err(()));
        assert_eq!(Imm::from_str(" 0 "), Err(()));
    }

    #[test]
    fn hex_imm() {
        assert_eq!(Imm::from_str("0x0"), Ok(Imm(0)));
        assert_eq!(Imm::from_str("0x1"), Ok(Imm(1)));
        assert_eq!(Imm::from_str("0x10"), Ok(Imm(16)));
        assert_eq!(Imm::from_str("0x00000100"), Ok(Imm(256)));
        assert_eq!(Imm::from_str("0x0000000A"), Ok(Imm(10)));
        assert_eq!(Imm::from_str("-0x0000000A"), Ok(Imm(-10)));
    }

    #[test]
    fn binary_imm() {
        assert_eq!(Imm::from_str("0b0"), Ok(Imm(0)));
        assert_eq!(Imm::from_str("0b1"), Ok(Imm(1)));
        assert_eq!(Imm::from_str("0b10"), Ok(Imm(2)));
        assert_eq!(Imm::from_str("0b00000010"), Ok(Imm(2)));
        assert_eq!(Imm::from_str("0b00000001"), Ok(Imm(1)));
        assert_eq!(Imm::from_str("0b00000000"), Ok(Imm(0)));
        assert_eq!(Imm::from_str("-0b00000000"), Ok(Imm(0)));
        assert_eq!(Imm::from_str("-0b00000001"), Ok(Imm(-1)));
        assert_eq!(Imm::from_str("-0b00000010"), Ok(Imm(-2)));
    }

    #[test]
    fn incorrect_pairings() {
        assert_eq!(Imm::from_str("0x"), Err(()));
        assert_eq!(Imm::from_str("0b"), Err(()));
        assert_eq!(Imm::from_str("0x-"), Err(()));
        assert_eq!(Imm::from_str("0b-"), Err(()));
        assert_eq!(Imm::from_str("0x-0"), Err(()));
        assert_eq!(Imm::from_str("0b-0"), Err(()));
        assert_eq!(Imm::from_str("0x-1"), Err(()));
        assert_eq!(Imm::from_str("0b-1"), Err(()));
        assert_eq!(Imm::from_str("0x-10"), Err(()));
        assert_eq!(Imm::from_str("0b-10"), Err(()));
        assert_eq!(Imm::from_str("0x-00000010"), Err(()));
        assert_eq!(Imm::from_str("0b-00000010"), Err(()));
        assert_eq!(Imm::from_str("0x-00000001"), Err(()));
        assert_eq!(Imm::from_str("0b-00000001"), Err(()));
        assert_eq!(Imm::from_str("0x-00000000"), Err(()));
        assert_eq!(Imm::from_str("0b-00000000"), Err(()));
    }
}
