use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign};

use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};

use crate::parser::RVRegister;

/// A set of registers that are used in a basic block.
///
/// This is currently limited to 32 registers as it is
/// tied heavily to the RISC-V architecture. For future
/// use cases, growing to u64 or u128 is a good option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    pub fn from_register(register: RVRegister) -> Self {
        let mut set = Self::new();
        set.set_register(&register);
        set
    }

    /// Set the given register in the set.
    pub fn set_register(&mut self, register: &RVRegister) {
        self.registers |= 1 << register.to_num();
    }

    /// Unset the given register in the set.
    pub fn unset_register(&mut self, register: &RVRegister) {
        self.registers &= !(1 << register.to_num());
    }

    /// Check if the given register is set in the set.
    #[must_use]
    pub fn contains(&self, register: &RVRegister) -> bool {
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
}

impl Default for RegisterSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for RegisterSetIter<'_> {
    type Item = RVRegister;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < 32 {
            let register = RVRegister::from_num(self.current).unwrap();
            self.current += 1;
            if self.registers.contains(&register) {
                return Some(register);
            }
        }
        None
    }
}

impl<'a> IntoIterator for &'a RegisterSet {
    type Item = RVRegister;
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

impl BitAnd<RVRegister> for RegisterSet {
    type Output = Self;

    fn bitand(self, rhs: RVRegister) -> Self::Output {
        Self {
            registers: self.registers & (1 << rhs.to_num()),
        }
    }
}

impl BitAndAssign<RVRegister> for RegisterSet {
    fn bitand_assign(&mut self, rhs: RVRegister) {
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

impl BitOr<RVRegister> for RegisterSet {
    type Output = Self;

    fn bitor(self, rhs: RVRegister) -> Self::Output {
        RegisterSet {
            registers: self.registers | (1 << rhs.to_num()),
        }
    }
}

impl BitOrAssign<RVRegister> for RegisterSet {
    fn bitor_assign(&mut self, rhs: RVRegister) {
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

impl Sub<RVRegister> for RegisterSet {
    type Output = Self;

    fn sub(self, rhs: RVRegister) -> Self::Output {
        Self {
            registers: self.registers & !(1 << rhs.to_num()),
        }
    }
}

impl SubAssign<RVRegister> for RegisterSet {
    fn sub_assign(&mut self, rhs: RVRegister) {
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
        let list = Vec::<RVRegister>::deserialize(deserializer)?;
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

impl FromIterator<RVRegister> for RegisterSet {
    fn from_iter<I: IntoIterator<Item = RVRegister>>(iter: I) -> Self {
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
        set.set_register(&RVRegister::X1);
        assert!(!set.is_empty());
        let mut set_iter = set.iter();
        assert_eq!(set_iter.next(), Some(RVRegister::X1));
        assert!(set_iter.next().is_none(), "Set should only contain X1");
        assert_eq!(set, [RVRegister::X1].into_iter().collect::<RegisterSet>());
    }

    #[test]
    fn can_loop_in_order_of_registers() {
        let mut set = RegisterSet::new();
        set.set_register(&RVRegister::X3);
        set |= RVRegister::X1;
        set |= RVRegister::X4;
        let mut set_iter = set.iter();
        assert_eq!(set_iter.next(), Some(RVRegister::X1));
        assert_eq!(set_iter.next(), Some(RVRegister::X3));
        assert_eq!(set_iter.next(), Some(RVRegister::X4));
        assert!(
            set_iter.next().is_none(),
            "Set should only contain X1, X2, X3"
        );
    }
}
