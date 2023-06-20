use std::str::FromStr;

use crate::parser::token::{Info, Token};

#[derive(Debug, PartialEq, Clone)]
pub struct Imm(pub i32);

impl TryFrom<Info> for Imm {
    type Error = ();

    fn try_from(value: Info) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => Imm::from_str(&s),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct CSRImm(pub u32);

impl TryFrom<Info> for CSRImm {
    type Error = ();

    fn try_from(value: Info) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => CSRImm::from_str(&s),
            _ => Err(()),
        }
    }
}

impl FromStr for CSRImm {
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
            _ => Imm::from_str(s)?.0 as u32,
        };
        Ok(CSRImm(num))
    }
}

impl FromStr for Imm {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.as_str();
        let s = s.trim();
        let neg = s.starts_with('-');
        let s = if neg { &s[1..] } else { s };
        let mul = if neg { -1 } else { 1 };

        if s == "zero" {
            Ok(Imm(0))
        } else if let Some(stripped) = s.strip_prefix("0x") {
            if stripped.starts_with('-') {
                Err(())
            } else {
                match u32::from_str_radix(stripped, 16) {
                    #[allow(clippy::cast_possible_wrap)]
                    Ok(i) => Ok(Imm(mul * i as i32)),
                    Err(_) => Err(()),
                }
            }
        } else if let Some(stripped) = s.strip_prefix("0b") {
            if stripped.starts_with('-') {
                Err(())
            } else {
                match u32::from_str_radix(stripped, 2) {
                    #[allow(clippy::cast_possible_wrap)]
                    Ok(i) => Ok(Imm(mul * i as i32)),
                    Err(_) => Err(()),
                }
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

impl From<Imm> for CSRImm {
    fn from(value: Imm) -> Self {
        #[allow(clippy::cast_sign_loss)]
        CSRImm(value.0 as u32)
    }
}

impl From<CSRImm> for Imm {
    fn from(value: CSRImm) -> Self {
        #[allow(clippy::cast_possible_wrap)]
        Imm(value.0 as i32)
    }
}

#[cfg(test)]
mod test {
    use crate::parser::imm::Imm;
    use std::str::FromStr;

    #[test]
    fn zero() {
        assert_eq!(Imm::from_str("zero"), Ok(Imm(0)));
        assert_eq!(Imm::from_str("ZERO"), Ok(Imm(0)));
    }

    #[test]
    fn basic_imm() {
        assert_eq!(Imm::from_str("0"), Ok(Imm(0)));
        assert_eq!(Imm::from_str("1"), Ok(Imm(1)));
        assert_eq!(Imm::from_str("-1"), Ok(Imm(-1)));
        assert_eq!(Imm::from_str("-16"), Ok(Imm(-16)));
    }

    #[test]
    fn neg_hex() {
        assert_eq!(Imm::from_str("0xFFFFFFFF"), Ok(Imm(-1)));
    }

    #[test]
    fn almost_neg_hex() {
        assert_eq!(Imm::from_str("0xFFFFFFFE"), Ok(Imm(-2)));
    }

    #[test]
    fn safe_hex() {
        assert_eq!(Imm::from_str("0x7FFFFFFF"), Ok(Imm(2_147_483_647)));
        assert_eq!(Imm::from_str("0x80000000"), Ok(Imm(-2_147_483_648)));
    }

    #[test]
    fn trim_allowed() {
        assert_eq!(Imm::from_str(" 120"), Ok(Imm(120)));
        assert_eq!(Imm::from_str("203 "), Ok(Imm(203)));
        assert_eq!(Imm::from_str(" 140 "), Ok(Imm(140)));
    }

    #[test]
    fn no_spaces_between() {
        assert_eq!(Imm::from_str("1 2"), Err(()));
        assert_eq!(Imm::from_str("1 2 3"), Err(()));
        assert_eq!(Imm::from_str("1 2 3 4"), Err(()));
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
