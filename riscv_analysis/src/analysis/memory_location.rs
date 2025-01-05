use serde::de::{self, Visitor};
use serde::{Deserialize, Serialize, Serializer};
use std::fmt;

/// A memory location.
///
/// This is used to represent a memory location.
///
/// Memory locations with relativity are always relative
/// from the entry point of a function. One consequence
/// is that multiple memory locations can have the same
/// offset. In other terms, equality across memory
/// contexts is not guaranteed.
///
/// The teminology used is as follows:
/// - `sp_i` is the stack pointer at the beginning of the function.
#[derive(Debug, PartialEq, Clone, Eq, Hash, PartialOrd, Ord)]
pub enum MemoryLocation {
    /// Offset from the stack pointer (sp) at the beginning of the function.
    ///
    /// The stack location offset is the offset added to the
    /// stack pointer to get a specific available value. For example,
    /// the integer `-8` will map to the value at address `sp - 8`.
    ///
    /// The stack pointer is always referring to the stack pointer
    /// value at the beginning of the function body.
    ///
    /// In the current implementation, only 32-bit values are
    /// kept track of on the stack. This is because the register
    /// is 32-bit.
    StackOffset(i32),
}

impl Serialize for MemoryLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MemoryLocation::StackOffset(i) => serializer.serialize_str(&format!(
                "so{}{}",
                if i < &0 { "-" } else { "+" },
                i.abs()
            )),
        }
    }
}

struct MemoryLocationVisitor;

impl Visitor<'_> for MemoryLocationVisitor {
    type Value = MemoryLocation;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a memory location custom representation")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if let Some(so) = v.strip_prefix("so") {
            let (sign, num) = so.split_at(1);
            let num = num.parse::<i32>().map_err(de::Error::custom)?;
            Ok(MemoryLocation::StackOffset(if sign == "-" {
                -num
            } else {
                num
            }))
        } else {
            Err(de::Error::custom("invalid memory location"))
        }
    }
}

impl<'de> Deserialize<'de> for MemoryLocation {
    fn deserialize<D>(deserializer: D) -> Result<MemoryLocation, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(MemoryLocationVisitor)
    }
}

// TODO: Introduce a "Function" field for `MemoryLocation::StackOffset`. This way, we
// can track the exact stack pointer offset for each function.

impl std::fmt::Display for MemoryLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // sp_i == "stack pointer at the beginning of the function, or initial"
        match self {
            MemoryLocation::StackOffset(offset) => {
                if offset < &0 {
                    write!(f, "sp_i - {}", offset.abs())
                } else {
                    write!(f, "sp_i + {offset}")
                }
            }
        }
    }
}
