use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::parser::token::{Info, Token};
use std::{
    collections::HashSet,
    convert::TryFrom,
    fmt::Display,
    hash::{Hash, Hasher},
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
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

impl TryFrom<Info> for Register {
    type Error = ();

    fn try_from(value: Info) -> Result<Self, Self::Error> {
        match value.token {
            Token::Symbol(s) => Register::from_str(&s),
            _ => Err(()),
        }
    }
}

impl FromStr for Register {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
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

impl Register {
    pub fn all_representations(&self) -> HashSet<String> {
        match self {
            Register::X0 => vec!["x0", "zero"],
            Register::X1 => vec!["x1", "ra"],
            Register::X2 => vec!["x2", "sp"],
            Register::X3 => vec!["x3", "gp"],
            Register::X4 => vec!["x4", "tp"],
            Register::X5 => vec!["x5", "t0"],
            Register::X6 => vec!["x6", "t1"],
            Register::X7 => vec!["x7", "t2"],
            Register::X8 => vec!["x8", "s0", "fp"],
            Register::X9 => vec!["x9", "s1"],
            Register::X10 => vec!["x10", "a0"],
            Register::X11 => vec!["x11", "a1"],
            Register::X12 => vec!["x12", "a2"],
            Register::X13 => vec!["x13", "a3"],
            Register::X14 => vec!["x14", "a4"],
            Register::X15 => vec!["x15", "a5"],
            Register::X16 => vec!["x16", "a6"],
            Register::X17 => vec!["x17", "a7"],
            Register::X18 => vec!["x18", "s2"],
            Register::X19 => vec!["x19", "s3"],
            Register::X20 => vec!["x20", "s4"],
            Register::X21 => vec!["x21", "s5"],
            Register::X22 => vec!["x22", "s6"],
            Register::X23 => vec!["x23", "s7"],
            Register::X24 => vec!["x24", "s8"],
            Register::X25 => vec!["x25", "s9"],
            Register::X26 => vec!["x26", "s10"],
            Register::X27 => vec!["x27", "s11"],
            Register::X28 => vec!["x28", "t3"],
            Register::X29 => vec!["x29", "t4"],
            Register::X30 => vec!["x30", "t5"],
            Register::X31 => vec!["x31", "t6"],
        }
        .iter()
        .copied()
        .map(std::string::ToString::to_string)
        .collect()
    }

    pub fn from_num(num: u8) -> Register {
        match num {
            0 => Register::X0,
            1 => Register::X1,
            2 => Register::X2,
            3 => Register::X3,
            4 => Register::X4,
            5 => Register::X5,
            6 => Register::X6,
            7 => Register::X7,
            8 => Register::X8,
            9 => Register::X9,
            10 => Register::X10,
            11 => Register::X11,
            12 => Register::X12,
            13 => Register::X13,
            14 => Register::X14,
            15 => Register::X15,
            16 => Register::X16,
            17 => Register::X17,
            18 => Register::X18,
            19 => Register::X19,
            20 => Register::X20,
            21 => Register::X21,
            22 => Register::X22,
            23 => Register::X23,
            24 => Register::X24,
            25 => Register::X25,
            26 => Register::X26,
            27 => Register::X27,
            28 => Register::X28,
            29 => Register::X29,
            30 => Register::X30,
            31 => Register::X31,
            _ => panic!("Invalid register number"),
        }
    }

    pub fn to_num(self) -> u8 {
        match self {
            Register::X0 => 0,
            Register::X1 => 1,
            Register::X2 => 2,
            Register::X3 => 3,
            Register::X4 => 4,
            Register::X5 => 5,
            Register::X6 => 6,
            Register::X7 => 7,
            Register::X8 => 8,
            Register::X9 => 9,
            Register::X10 => 10,
            Register::X11 => 11,
            Register::X12 => 12,
            Register::X13 => 13,
            Register::X14 => 14,
            Register::X15 => 15,
            Register::X16 => 16,
            Register::X17 => 17,
            Register::X18 => 18,
            Register::X19 => 19,
            Register::X20 => 20,
            Register::X21 => 21,
            Register::X22 => 22,
            Register::X23 => 23,
            Register::X24 => 24,
            Register::X25 => 25,
            Register::X26 => 26,
            Register::X27 => 27,
            Register::X28 => 28,
            Register::X29 => 29,
            Register::X30 => 30,
            Register::X31 => 31,
        }
    }

    pub fn is_sp(self) -> bool {
        self == Register::X2
    }

    pub fn ecall_type() -> Register {
        Register::X17
    }

    pub fn all() -> HashSet<Register> {
        vec![
            Register::X0,
            Register::X1,
            Register::X2,
            Register::X3,
            Register::X4,
            Register::X5,
            Register::X6,
            Register::X7,
            Register::X8,
            Register::X9,
            Register::X10,
            Register::X11,
            Register::X12,
            Register::X13,
            Register::X14,
            Register::X15,
            Register::X16,
            Register::X17,
            Register::X18,
            Register::X19,
            Register::X20,
            Register::X21,
            Register::X22,
            Register::X23,
            Register::X24,
            Register::X25,
            Register::X26,
            Register::X27,
            Register::X28,
            Register::X29,
            Register::X30,
            Register::X31,
        ]
        .iter()
        .copied()
        .collect()
    }
}

impl Hash for Register {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_num().hash(state);
    }
}
impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Register::{
            X0, X1, X10, X11, X12, X13, X14, X15, X16, X17, X18, X19, X2, X20, X21, X22, X23, X24,
            X25, X26, X27, X28, X29, X3, X30, X31, X4, X5, X6, X7, X8, X9,
        };
        let res = match self {
            X0 => "zero",
            X1 => "ra",
            X2 => "sp",
            X3 => "gp",
            X4 => "tp",
            X5 => "t0",
            X6 => "t1",
            X7 => "t2",
            X8 => "s0",
            X9 => "s1",
            X10 => "a0",
            X11 => "a1",
            X12 => "a2",
            X13 => "a3",
            X14 => "a4",
            X15 => "a5",
            X16 => "a6",
            X17 => "a7",
            X18 => "s2",
            X19 => "s3",
            X20 => "s4",
            X21 => "s5",
            X22 => "s6",
            X23 => "s7",
            X24 => "s8",
            X25 => "s9",
            X26 => "s10",
            X27 => "s11",
            X28 => "t3",
            X29 => "t4",
            X30 => "t5",
            X31 => "t6",
        };
        f.write_str(res)
    }
}
