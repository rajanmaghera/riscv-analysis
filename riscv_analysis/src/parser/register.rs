use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{cfg::RegisterSet, parser::token::RVToken};
use std::{
    collections::HashSet,
    fmt::Display,
    hash::{Hash, Hasher},
    str::FromStr,
};

use super::{TokenType, With};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum RVRegister {
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

impl TryFrom<RVToken> for RVRegister {
    type Error = ();

    fn try_from(value: RVToken) -> Result<Self, Self::Error> {
        match value.token_type() {
            TokenType::Symbol(s) => RVRegister::from_str(s),
            _ => Err(()),
        }
    }
}

impl FromStr for RVRegister {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x0" | "zero" => Ok(RVRegister::X0),
            "x1" | "ra" => Ok(RVRegister::X1),
            "x2" | "sp" => Ok(RVRegister::X2),
            "x3" | "gp" => Ok(RVRegister::X3),
            "x4" | "tp" => Ok(RVRegister::X4),
            "x5" | "t0" => Ok(RVRegister::X5),
            "x6" | "t1" => Ok(RVRegister::X6),
            "x7" | "t2" => Ok(RVRegister::X7),
            "x8" | "s0" | "fp" => Ok(RVRegister::X8),
            "x9" | "s1" => Ok(RVRegister::X9),
            "x10" | "a0" => Ok(RVRegister::X10),
            "x11" | "a1" => Ok(RVRegister::X11),
            "x12" | "a2" => Ok(RVRegister::X12),
            "x13" | "a3" => Ok(RVRegister::X13),
            "x14" | "a4" => Ok(RVRegister::X14),
            "x15" | "a5" => Ok(RVRegister::X15),
            "x16" | "a6" => Ok(RVRegister::X16),
            "x17" | "a7" => Ok(RVRegister::X17),
            "x18" | "s2" => Ok(RVRegister::X18),
            "x19" | "s3" => Ok(RVRegister::X19),
            "x20" | "s4" => Ok(RVRegister::X20),
            "x21" | "s5" => Ok(RVRegister::X21),
            "x22" | "s6" => Ok(RVRegister::X22),
            "x23" | "s7" => Ok(RVRegister::X23),
            "x24" | "s8" => Ok(RVRegister::X24),
            "x25" | "s9" => Ok(RVRegister::X25),
            "x26" | "s10" => Ok(RVRegister::X26),
            "x27" | "s11" => Ok(RVRegister::X27),
            "x28" | "t3" => Ok(RVRegister::X28),
            "x29" | "t4" => Ok(RVRegister::X29),
            "x30" | "t5" => Ok(RVRegister::X30),
            "x31" | "t6" => Ok(RVRegister::X31),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct ParseRegisterError;

impl Display for ParseRegisterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid register")
    }
}

impl RVRegister {
    #[must_use]
    pub fn all_representations(&self) -> HashSet<String> {
        match self {
            RVRegister::X0 => vec!["x0", "zero"],
            RVRegister::X1 => vec!["x1", "ra"],
            RVRegister::X2 => vec!["x2", "sp"],
            RVRegister::X3 => vec!["x3", "gp"],
            RVRegister::X4 => vec!["x4", "tp"],
            RVRegister::X5 => vec!["x5", "t0"],
            RVRegister::X6 => vec!["x6", "t1"],
            RVRegister::X7 => vec!["x7", "t2"],
            RVRegister::X8 => vec!["x8", "s0", "fp"],
            RVRegister::X9 => vec!["x9", "s1"],
            RVRegister::X10 => vec!["x10", "a0"],
            RVRegister::X11 => vec!["x11", "a1"],
            RVRegister::X12 => vec!["x12", "a2"],
            RVRegister::X13 => vec!["x13", "a3"],
            RVRegister::X14 => vec!["x14", "a4"],
            RVRegister::X15 => vec!["x15", "a5"],
            RVRegister::X16 => vec!["x16", "a6"],
            RVRegister::X17 => vec!["x17", "a7"],
            RVRegister::X18 => vec!["x18", "s2"],
            RVRegister::X19 => vec!["x19", "s3"],
            RVRegister::X20 => vec!["x20", "s4"],
            RVRegister::X21 => vec!["x21", "s5"],
            RVRegister::X22 => vec!["x22", "s6"],
            RVRegister::X23 => vec!["x23", "s7"],
            RVRegister::X24 => vec!["x24", "s8"],
            RVRegister::X25 => vec!["x25", "s9"],
            RVRegister::X26 => vec!["x26", "s10"],
            RVRegister::X27 => vec!["x27", "s11"],
            RVRegister::X28 => vec!["x28", "t3"],
            RVRegister::X29 => vec!["x29", "t4"],
            RVRegister::X30 => vec!["x30", "t5"],
            RVRegister::X31 => vec!["x31", "t6"],
        }
        .iter()
        .copied()
        .map(std::string::ToString::to_string)
        .collect()
    }

    /// Returns a register from a number
    pub fn from_num(num: u8) -> Result<RVRegister, ParseRegisterError> {
        Ok(match num {
            0 => RVRegister::X0,
            1 => RVRegister::X1,
            2 => RVRegister::X2,
            3 => RVRegister::X3,
            4 => RVRegister::X4,
            5 => RVRegister::X5,
            6 => RVRegister::X6,
            7 => RVRegister::X7,
            8 => RVRegister::X8,
            9 => RVRegister::X9,
            10 => RVRegister::X10,
            11 => RVRegister::X11,
            12 => RVRegister::X12,
            13 => RVRegister::X13,
            14 => RVRegister::X14,
            15 => RVRegister::X15,
            16 => RVRegister::X16,
            17 => RVRegister::X17,
            18 => RVRegister::X18,
            19 => RVRegister::X19,
            20 => RVRegister::X20,
            21 => RVRegister::X21,
            22 => RVRegister::X22,
            23 => RVRegister::X23,
            24 => RVRegister::X24,
            25 => RVRegister::X25,
            26 => RVRegister::X26,
            27 => RVRegister::X27,
            28 => RVRegister::X28,
            29 => RVRegister::X29,
            30 => RVRegister::X30,
            31 => RVRegister::X31,
            _ => return Err(ParseRegisterError),
        })
    }

    #[must_use]
    pub fn to_num(self) -> u8 {
        match self {
            RVRegister::X0 => 0,
            RVRegister::X1 => 1,
            RVRegister::X2 => 2,
            RVRegister::X3 => 3,
            RVRegister::X4 => 4,
            RVRegister::X5 => 5,
            RVRegister::X6 => 6,
            RVRegister::X7 => 7,
            RVRegister::X8 => 8,
            RVRegister::X9 => 9,
            RVRegister::X10 => 10,
            RVRegister::X11 => 11,
            RVRegister::X12 => 12,
            RVRegister::X13 => 13,
            RVRegister::X14 => 14,
            RVRegister::X15 => 15,
            RVRegister::X16 => 16,
            RVRegister::X17 => 17,
            RVRegister::X18 => 18,
            RVRegister::X19 => 19,
            RVRegister::X20 => 20,
            RVRegister::X21 => 21,
            RVRegister::X22 => 22,
            RVRegister::X23 => 23,
            RVRegister::X24 => 24,
            RVRegister::X25 => 25,
            RVRegister::X26 => 26,
            RVRegister::X27 => 27,
            RVRegister::X28 => 28,
            RVRegister::X29 => 29,
            RVRegister::X30 => 30,
            RVRegister::X31 => 31,
        }
    }

    #[must_use]
    pub fn ecall_type() -> RVRegister {
        RVRegister::X17
    }

    #[must_use]
    pub fn all() -> RegisterSet {
        [
            RVRegister::X0,
            RVRegister::X1,
            RVRegister::X2,
            RVRegister::X3,
            RVRegister::X4,
            RVRegister::X5,
            RVRegister::X6,
            RVRegister::X7,
            RVRegister::X8,
            RVRegister::X9,
            RVRegister::X10,
            RVRegister::X11,
            RVRegister::X12,
            RVRegister::X13,
            RVRegister::X14,
            RVRegister::X15,
            RVRegister::X16,
            RVRegister::X17,
            RVRegister::X18,
            RVRegister::X19,
            RVRegister::X20,
            RVRegister::X21,
            RVRegister::X22,
            RVRegister::X23,
            RVRegister::X24,
            RVRegister::X25,
            RVRegister::X26,
            RVRegister::X27,
            RVRegister::X28,
            RVRegister::X29,
            RVRegister::X30,
            RVRegister::X31,
        ]
        .iter()
        .copied()
        .collect()
    }
}

impl Hash for RVRegister {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_num().hash(state);
    }
}
impl Display for RVRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let res = match self {
            RVRegister::X0 => "zero",
            RVRegister::X1 => "ra",
            RVRegister::X2 => "sp",
            RVRegister::X3 => "gp",
            RVRegister::X4 => "tp",
            RVRegister::X5 => "t0",
            RVRegister::X6 => "t1",
            RVRegister::X7 => "t2",
            RVRegister::X8 => "s0",
            RVRegister::X9 => "s1",
            RVRegister::X10 => "a0",
            RVRegister::X11 => "a1",
            RVRegister::X12 => "a2",
            RVRegister::X13 => "a3",
            RVRegister::X14 => "a4",
            RVRegister::X15 => "a5",
            RVRegister::X16 => "a6",
            RVRegister::X17 => "a7",
            RVRegister::X18 => "s2",
            RVRegister::X19 => "s3",
            RVRegister::X20 => "s4",
            RVRegister::X21 => "s5",
            RVRegister::X22 => "s6",
            RVRegister::X23 => "s7",
            RVRegister::X24 => "s8",
            RVRegister::X25 => "s9",
            RVRegister::X26 => "s10",
            RVRegister::X27 => "s11",
            RVRegister::X28 => "t3",
            RVRegister::X29 => "t4",
            RVRegister::X30 => "t5",
            RVRegister::X31 => "t6",
        };
        f.write_str(res)
    }
}

impl RVRegister {
    #[must_use]
    pub fn stack_pointer() -> Self {
        RVRegister::X2
    }
}

pub type RegisterToken = With<RVRegister>;
