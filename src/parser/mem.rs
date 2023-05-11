use std::str::FromStr;

use crate::parser::imm::Imm;
use crate::parser::register::Register;
use crate::parser::token::{Position, Range, SymbolData, Token, TokenInfo, WithToken};
// Used for parsing 0(t2) type expressions

#[derive(Debug, PartialEq, Clone)]
pub struct Mem {
    pub offset: WithToken<Imm>,
    pub reg: WithToken<Register>,
}

impl TryFrom<TokenInfo> for Mem {
    type Error = ();

    fn try_from(value: TokenInfo) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => {
                let mut split = s.0.split('(');
                let offset = split.next().ok_or(())?;
                let reg = split.next().ok_or(())?;
                let reg = reg.trim().trim_end_matches(')');
                let reg_raw = Register::from_str(reg.trim())?;

                // if offset_raw is all whitespace, then it is 0
                let offset_raw = if offset.trim() == "" {
                    Imm(0)
                } else {
                    Imm::from_str(offset.trim())?
                };

                // calculate positions of tokens
                // TODO fix spacing issue ex. 0(t2) vs 0 ( t2 )
                // TODO move to token as helper functions
                let offset_start = value.pos.start;
                let offset_end = Position {
                    line: offset_start.line,
                    column: offset_start.column + offset.len(),
                };
                let reg_start = Position {
                    line: offset_start.line,
                    column: offset_end.column + 1,
                };
                let reg_end = Position {
                    line: reg_start.line,
                    column: reg_start.column + reg.len(),
                };

                let offset = WithToken {
                    token: Token::Symbol(SymbolData(offset.to_string())),
                    pos: Range {
                        start: offset_start,
                        end: offset_end,
                    },
                    data: offset_raw,
                };
                let reg = WithToken {
                    token: Token::Symbol(SymbolData(reg.to_string())),
                    pos: Range {
                        start: reg_start,
                        end: reg_end,
                    },
                    data: reg_raw,
                };

                Ok(Mem { offset, reg })
            }
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod test {

    // macro rules for testing
    macro_rules! token {
        ($x:expr) => {
            TokenInfo {
                token: Token::Symbol(SymbolData($x.to_owned())),
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

    macro_rules! act {
        ($x:expr) => {
            Mem::try_from(token!($x)).unwrap()
        };
    }

    macro_rules! exp {
        ($a:expr, $b:ident) => {
            Mem {
                offset: WithToken::blank(Imm($a)),
                reg: WithToken::blank(Register::$b),
            }
        };
    }
    use super::*;
    use crate::parser::token::Position;

    #[test]
    fn bad_spacing() {
        assert_eq!(exp!(0, X2), act!("  0(sp)  "));
        assert_eq!(exp!(0, X2), act!("  0 (sp)  "));
        assert_eq!(exp!(0, X2), act!("  0 ( sp)  "));
        assert_eq!(exp!(0, X2), act!("  0 ( sp )  "));
        assert_eq!(exp!(0, X2), act!("  0 (sp )  "));
        assert_eq!(exp!(0, X2), act!("  0(sp )  "));
        assert_eq!(exp!(0, X2), act!("  0( sp)  "));
        assert_eq!(exp!(0, X2), act!("   ( sp)  "));
        assert_eq!(exp!(0, X2), act!("   ( sp )  "));
        assert_eq!(exp!(0, X2), act!("   (sp )  "));
    }

    #[test]
    fn no_spacing_between() {
        assert!(Mem::try_from(token!("0(s p)").to_owned()).is_err());
        assert!(Mem::try_from(token!("1 2 (sp)").to_owned()).is_err());
        assert!(Mem::try_from(token!("(x 0)").to_owned()).is_err());

    }

    #[test]
    fn basic_test() {
        assert_eq!(exp!(0, X2), act!("0(sp)"));
        assert_eq!(exp!(4, X8), act!("4(x8)"));
        assert_eq!(exp!(-8, X0), act!("-8(zero)"));
    }

    #[test]
    fn hex_imm() {
        assert_eq!(exp!(0, X2), act!("0x00(x2)"));
        assert_eq!(exp!(16, X8), act!("0x010(x8)"));
        assert_eq!(exp!(-16, X0), act!("-0x0010(x0)"));
    }

    #[test]
    fn bin_imm() {
        assert_eq!(exp!(0, X2), act!("0b00(x2)"));
        assert_eq!(exp!(16, X8), act!("0b010000(x8)"));
        assert_eq!(exp!(-16, X0), act!("-0b0000000000010000(x0)"));
    }

    #[test]
    fn missing_imm() {
        assert_eq!(exp!(0, X2), act!("(x2)"));
        assert_eq!(exp!(0, X0), act!("(zero)"));
        assert_eq!(exp!(0, X0), act!("(x0)"));
    }

    #[test]
    fn fail_immediate_on_own() {
        assert!(Mem::try_from(token!("0").to_owned()).is_err());
    }
}
