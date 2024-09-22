use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign};

use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};

use crate::{analysis::AvailableValue, parser::Register};

use super::AvailableValueMap;

/// A set of registers that are used in a basic block.
///
/// This is currently limited to 32 registers as it is
/// tied heavily to the RISC-V architecture. For future
/// use cases, growing to u64 or u128 is a good option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterSet {
    /// The registers that are used in the basic block.
    /// The bit at index `i` is set if register `i` is used.
    /// For example, the number 0x00000003 would indicate
    /// that registers X0 and X1 are used.
    registers: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterSetIter<'a> {
    registers: &'a RegisterSet,
    current: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedRegisterSetIter {
    registers: RegisterSet,
    current: u8,
}

pub trait LegacyRegisterSet {}

impl RegisterSet {
    /// Create a new `RegisterSet` with no registers set.
    pub fn new() -> Self {
        Self { registers: 0 }
    }

    /// Create a new `RegisterSet` from a list of registers.
    pub fn from_registers<'a, I>(registers: I) -> Self
    where
        I: IntoIterator<Item = &'a Register>,
    {
        let mut set = Self::new();
        for register in registers {
            set.set_register(register);
        }
        set
    }

    /// Create a new `RegisterSet` from a single register.
    pub fn from_register(register: Register) -> Self {
        Self::from_registers(&[register])
    }

    /// Set the given register in the set.
    pub fn set_register(&mut self, register: &Register) {
        self.registers |= 1 << register.to_num();
    }

    /// Unset the given register in the set.
    pub fn unset_register(&mut self, register: &Register) {
        self.registers &= !(1 << register.to_num());
    }

    /// Check if the given register is set in the set.
    pub fn contains(&self, register: &Register) -> bool {
        self.registers & (1 << register.to_num()) != 0
    }

    /// Return a borrowed iterator.
    pub fn iter(&self) -> RegisterSetIter<'_> {
        self.into_iter()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.registers == 0
    }

    /// Represent the set of registers as a map to available values, with
    /// all the registers set to their original value.
    pub fn into_available_values(self) -> AvailableValueMap<Register> {
        self.into_iter()
            .map(|x| (x, AvailableValue::OriginalRegisterWithScalar(x, 0)))
            .collect()
    }
}

impl Default for RegisterSet {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Iterator for RegisterSetIter<'a> {
    type Item = Register;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < 32 {
            let register = Register::from_num(self.current).unwrap();
            self.current += 1;
            if self.registers.contains(&register) {
                return Some(register);
            }
        }
        None
    }
}

impl<'a> IntoIterator for &'a RegisterSet {
    type Item = Register;
    type IntoIter = RegisterSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RegisterSetIter {
            registers: &self,
            current: 0,
        }
    }
}

impl BitAnd for RegisterSet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            registers: self.registers & rhs.registers,
        }
    }
}

impl BitAndAssign for RegisterSet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.registers &= rhs.registers;
    }
}

impl BitAnd<Register> for RegisterSet {
    type Output = Self;

    fn bitand(self, rhs: Register) -> Self::Output {
        Self {
            registers: self.registers & (1 << rhs.to_num()),
        }
    }
}

impl BitAndAssign<Register> for RegisterSet {
    fn bitand_assign(&mut self, rhs: Register) {
        self.registers &= 1 << rhs.to_num();
    }
}

impl BitOr for RegisterSet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            registers: self.registers | rhs.registers,
        }
    }
}

impl BitOrAssign for RegisterSet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.registers |= rhs.registers;
    }
}

impl BitOr<Register> for RegisterSet {
    type Output = Self;

    fn bitor(self, rhs: Register) -> Self::Output {
        RegisterSet {
            registers: self.registers | (1 << rhs.to_num()),
        }
    }
}

impl BitOrAssign<Register> for RegisterSet {
    fn bitor_assign(&mut self, rhs: Register) {
        self.registers |= 1 << rhs.to_num();
    }
}

impl Sub for RegisterSet {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            registers: self.registers & !rhs.registers,
        }
    }
}

impl SubAssign for RegisterSet {
    fn sub_assign(&mut self, rhs: Self) {
        self.registers &= !rhs.registers;
    }
}

impl Sub<Register> for RegisterSet {
    type Output = Self;

    fn sub(self, rhs: Register) -> Self::Output {
        Self {
            registers: self.registers & !(1 << rhs.to_num()),
        }
    }
}

impl SubAssign<Register> for RegisterSet {
    fn sub_assign(&mut self, rhs: Register) {
        self.registers &= !(1 << rhs.to_num());
    }
}

impl std::fmt::Display for RegisterSet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut first = true;
        write!(f, "[")?;
        for register in self {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{}", register)?;
        }
        write!(f, "]")
    }
}

impl<'a> Deserialize<'a> for RegisterSet {
    fn deserialize<D>(deserializer: D) -> Result<RegisterSet, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let list = Vec::<Register>::deserialize(deserializer)?;
        Ok(RegisterSet::from_registers(list.iter()))
    }
}

impl Serialize for RegisterSet {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.iter()
            .sorted()
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}

impl FromIterator<Register> for RegisterSet {
    fn from_iter<I: IntoIterator<Item = Register>>(iter: I) -> Self {
        let mut set = Self::new();
        for register in iter {
            set.set_register(&register);
        }
        set
    }
}
