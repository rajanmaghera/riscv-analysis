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
                let reg = reg.trim_end_matches(')');
                let reg_raw = Register::from_str(reg)?;
                let offset_raw = Imm::from_str(offset)?;

                // calculate positions of tokens
                // TODO fix spacing issue ex. 0(t2) vs 0 ( t2 )
                // TODO move to token as helper functions
                // TODO test
                // TODO MISSING IMMEDIATE NUMBER!!!
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
