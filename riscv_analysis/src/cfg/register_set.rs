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

#[derive(Debug, Clone)]
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
    #[must_use]
    pub fn new() -> Self {
        Self { registers: 0 }
    }

    /// Create a new `RegisterSet` from a single register.
    #[must_use]
    pub fn from_register(register: Register) -> Self {
        let mut set = Self::new();
        set.set_register(&register);
        set
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
    #[must_use]
    pub fn contains(&self, register: &Register) -> bool {
        self.registers & (1 << register.to_num()) != 0
    }

    /// Return a borrowed iterator.
    #[must_use]
    pub fn iter(&self) -> RegisterSetIter<'_> {
        self.into_iter()
    }

    /// Check if the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.registers == 0
    }

    /// Represent the set of registers as a map to available values, with
    /// all the registers set to their original value.
    #[must_use]
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

impl Iterator for RegisterSetIter<'_> {
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
            registers: self,
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
            write!(f, "{register}")?;
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
        Ok(list.into_iter().collect())
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn can_create_an_empty_set() {
        let set = RegisterSet::new();
        assert!(set.is_empty());
        let mut set_iter = set.iter();
        assert!(set_iter.next().is_none(), "Set should be empty");
    }

    #[test]
    fn can_use_set_of_one_register() {
        let mut set = RegisterSet::new();
        set.set_register(&Register::X1);
        assert!(!set.is_empty());
        let mut set_iter = set.iter();
        assert_eq!(set_iter.next(), Some(Register::X1));
        assert!(set_iter.next().is_none(), "Set should only contain X1");
        assert_eq!(set, [Register::X1].into_iter().collect::<RegisterSet>());
    }

    #[test]
    fn can_loop_in_order_of_registers() {
        let mut set = RegisterSet::new();
        set.set_register(&Register::X3);
        set |= Register::X1;
        set |= Register::X2;
        let mut set_iter = set.iter();
        assert_eq!(set_iter.next(), Some(Register::X1));
        assert_eq!(set_iter.next(), Some(Register::X2));
        assert_eq!(set_iter.next(), Some(Register::X3));
        assert!(
            set_iter.next().is_none(),
            "Set should only contain X1, X2, X3"
        );
    }
}
