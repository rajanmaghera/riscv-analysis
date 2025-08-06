use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::parser::token::Token;

use super::TokenType;

/// Descriptor of immediate types
///
/// In some cases, the immediate is a reference
/// to a label. In that case, the immediate is
/// not known and is treated as an "any" value
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
enum ImmType {
    Unknown,
    Constant(i32),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Imm(ImmType);

impl Imm {
    #[must_use]
    pub fn new(value: i32) -> Self {
        Imm(ImmType::Constant(value))
    }

    pub fn new_unknown() -> Self {
        Imm(ImmType::Unknown)
    }

    #[must_use]
    pub fn value(&self) -> Option<i32> {
        match self.0 {
            ImmType::Constant(c) => Some(c),
            ImmType::Unknown => None,
        }
    }
}

impl TryFrom<Token> for Imm {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.token_type() {
            TokenType::Symbol(s) => Imm::from_str(s),
            TokenType::Char(c) => Ok(Imm(ImmType::Constant(*c as i32))),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Clone, Deserialize, Serialize)]
pub struct CsrImm(u32);

impl CsrImm {
    #[must_use]
    pub fn new(value: u32) -> Self {
        CsrImm(value)
    }

    #[must_use]
    pub fn value(&self) -> u32 {
        self.0
    }
}

impl TryFrom<Token> for CsrImm {
    type Error = ();

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value.token_type() {
            TokenType::Symbol(s) => CsrImm::from_str(s),
            _ => Err(()),
        }
    }
}

impl FromStr for CsrImm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let string = s.to_lowercase();
        let num = match string.as_str() {
            "ustatus" => 0x000,
            "fflags" => 0x001,
            "frm" => 0x002,
            "fcsr" => 0x003,
            "uie" => 0x004,
            "utvec" => 0x005,
            "uscratch" => 0x040,
            "uepc" => 0x041,
            "ucause" => 0x042,
            "utval" => 0x043,
            "uip" => 0x044,
            "cycle" => 0xC00,
            "time" => 0xC01,
            "instret" => 0xC02,
            "cycleh" => 0xC80,
            "timeh" => 0xC81,
            "instreth" => 0xC82,
            #[allow(clippy::cast_sign_loss)]
            _ => Imm::from_str(s)?.value().ok_or(())? as u32,
        };
        Ok(CsrImm(num))
    }
}

impl FromStr for Imm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.as_str();
        let s = s.trim();
        let (s, mul) = if let Some(stripped) = s.strip_prefix('-') {
            (stripped, -1)
        } else {
            (s, 1)
        };

        if s == "zero" {
            Ok(Imm::new(0))
        } else if let Some(stripped) = s.strip_prefix("0x") {
            if stripped.starts_with('-') {
                Err(())
            } else {
                match u32::from_str_radix(stripped, 16) {
                    #[allow(clippy::cast_possible_wrap)]
                    Ok(i) => Ok(Imm::new(mul * i as i32)),
                    Err(_) => Err(()),
                }
            }
        } else if let Some(stripped) = s.strip_prefix("0b") {
            if stripped.starts_with('-') {
                Err(())
            } else {
                match u32::from_str_radix(stripped, 2) {
                    #[allow(clippy::cast_possible_wrap)]
                    Ok(i) => Ok(Imm::new(mul * i as i32)),
                    Err(_) => Err(()),
                }
            }
        } else {
            if s.starts_with('-') {
                return Err(());
            }
            match s.parse::<i32>() {
                Ok(i) => Ok(Imm::new(mul * i)),
                Err(_) => Err(()),
            }
        }
    }
}

impl From<Imm> for CsrImm {
    fn from(value: Imm) -> Self {
        match value.0 {
            ImmType::Unknown => panic!(),
            #[allow(clippy::cast_sign_loss)]
            ImmType::Constant(x) => CsrImm(x as u32),
        }
    }
}

impl From<CsrImm> for Imm {
    fn from(value: CsrImm) -> Self {
        #[allow(clippy::cast_possible_wrap)]
        Imm::new(value.0 as i32)
    }
}

impl std::fmt::Display for Imm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            ImmType::Constant(c) => write!(f, "{}", c),
            ImmType::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::imm::Imm;
    use std::str::FromStr;

    #[test]
    fn zero() {
        assert_eq!(Imm::from_str("zero"), Ok(Imm::new(0)));
        assert_eq!(Imm::from_str("ZERO"), Ok(Imm::new(0)));
    }

    #[test]
    fn basic_imm() {
        assert_eq!(Imm::from_str("0"), Ok(Imm::new(0)));
        assert_eq!(Imm::from_str("1"), Ok(Imm::new(1)));
        assert_eq!(Imm::from_str("-1"), Ok(Imm::new(-1)));
        assert_eq!(Imm::from_str("-16"), Ok(Imm::new(-16)));
    }

    #[test]
    fn neg_hex() {
        assert_eq!(Imm::from_str("0xFFFFFFFF"), Ok(Imm::new(-1)));
    }

    #[test]
    fn almost_neg_hex() {
        assert_eq!(Imm::from_str("0xFFFFFFFE"), Ok(Imm::new(-2)));
    }

    #[test]
    fn safe_hex() {
        assert_eq!(Imm::from_str("0x7FFFFFFF"), Ok(Imm::new(0x7FFF_FFFF)));
        assert_eq!(Imm::from_str("0x80000000"), Ok(Imm::new(-0x8000_0000)));
    }

    #[test]
    fn trim_allowed() {
        assert_eq!(Imm::from_str(" 120"), Ok(Imm::new(120)));
        assert_eq!(Imm::from_str("203 "), Ok(Imm::new(203)));
        assert_eq!(Imm::from_str(" 140 "), Ok(Imm::new(140)));
    }

    #[test]
    fn no_spaces_between() {
        assert_eq!(Imm::from_str("1 2"), Err(()));
        assert_eq!(Imm::from_str("1 2 3"), Err(()));
        assert_eq!(Imm::from_str("1 2 3 4"), Err(()));
    }

    #[test]
    fn hex_imm() {
        assert_eq!(Imm::from_str("0x0"), Ok(Imm::new(0)));
        assert_eq!(Imm::from_str("0x1"), Ok(Imm::new(1)));
        assert_eq!(Imm::from_str("0x10"), Ok(Imm::new(16)));
        assert_eq!(Imm::from_str("0x00000100"), Ok(Imm::new(256)));
        assert_eq!(Imm::from_str("0x0000000A"), Ok(Imm::new(10)));
        assert_eq!(Imm::from_str("-0x0000000A"), Ok(Imm::new(-10)));
    }

    #[test]
    fn binary_imm() {
        assert_eq!(Imm::from_str("0b0"), Ok(Imm::new(0)));
        assert_eq!(Imm::from_str("0b1"), Ok(Imm::new(1)));
        assert_eq!(Imm::from_str("0b10"), Ok(Imm::new(2)));
        assert_eq!(Imm::from_str("0b00000010"), Ok(Imm::new(2)));
        assert_eq!(Imm::from_str("0b00000001"), Ok(Imm::new(1)));
        assert_eq!(Imm::from_str("0b00000000"), Ok(Imm::new(0)));
        assert_eq!(Imm::from_str("-0b00000000"), Ok(Imm::new(0)));
        assert_eq!(Imm::from_str("-0b00000001"), Ok(Imm::new(-1)));
        assert_eq!(Imm::from_str("-0b00000010"), Ok(Imm::new(-2)));
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
