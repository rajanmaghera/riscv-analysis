//! Floating-point (F and D extension) instructions.
//!
//! RVA does not model the floating-point registers. A floating-point
//! instruction is kept only for what it does to the integer registers and to
//! memory: loads and stores read their base register, conversions and moves
//! read or write one integer register, and comparisons write one. Everything
//! that happens in the floating-point registers is ignored.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Imm, LabelStringToken, RVRegister, RawToken, With};
use crate::cfg::Segment;

/// What a floating-point instruction does with the integer registers and memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FloatKind {
    /// `flw`/`fld fd, imm(rs1)`: reads `rs1` and loads from memory.
    Load,
    /// `fsw`/`fsd fs2, imm(rs1)`: reads `rs1` and stores to memory.
    Store,
    /// For example `fcvt.d.w fd, rs1`: reads the integer register `rs1`.
    FromInt,
    /// For example `fcvt.w.d rd, fs1` or `flt.d rd, fs1, fs2`: writes the
    /// integer register `rd`.
    ToInt,
    /// Reads and writes only floating-point registers.
    Compute,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Float {
    /// The mnemonic, such as `fmadd.d`.
    pub inst: With<String>,
    pub kind: FloatKind,
    /// The integer register written, for [`FloatKind::ToInt`].
    pub rd: Option<With<RVRegister>>,
    /// The integer register read: the base address of a load or store, or
    /// the source of a [`FloatKind::FromInt`] instruction.
    pub rs1: Option<With<RVRegister>>,
    /// The offset of a load or store.
    pub imm: Option<With<Imm>>,
    /// The number of bytes a load or store accesses.
    pub width: i32,
    #[serde(skip)]
    pub key: Uuid,
    #[serde(skip, default = "super::details::empty_token")]
    pub token: RawToken,
    pub segment: Segment,
    pub labels: HashSet<LabelStringToken>,
}

/// Classify a floating-point mnemonic, or return `None` if `inst` is not one.
#[must_use]
pub fn float_kind(inst: &str) -> Option<FloatKind> {
    let inst = inst.to_lowercase();
    let (op, fmt) = inst.rsplit_once('.')?;
    let single_or_double = matches!(fmt, "s" | "d");
    let kind = match (op, fmt) {
        ("fcvt.w" | "fcvt.wu", "s" | "d") | ("fmv.x", "w" | "s") => FloatKind::ToInt,
        ("fcvt.s" | "fcvt.d", "w" | "wu") => FloatKind::FromInt,
        ("fmv.w" | "fmv.s", "x") => FloatKind::FromInt,
        ("fcvt.s", "d") | ("fcvt.d", "s") => FloatKind::Compute,
        ("feq" | "flt" | "fle" | "fclass", _) if single_or_double => FloatKind::ToInt,
        (
            "fadd" | "fsub" | "fmul" | "fdiv" | "fsqrt" | "fmin" | "fmax" | "fsgnj" | "fsgnjn"
            | "fsgnjx" | "fmadd" | "fmsub" | "fnmadd" | "fnmsub" | "fmv" | "fneg" | "fabs",
            _,
        ) if single_or_double => FloatKind::Compute,
        _ => return None,
    };
    Some(kind)
}

/// Classify a floating-point load or store mnemonic and return its width in bytes.
#[must_use]
pub fn float_memory_kind(inst: &str) -> Option<(FloatKind, i32)> {
    match inst.to_lowercase().as_str() {
        "flw" => Some((FloatKind::Load, 4)),
        "fld" => Some((FloatKind::Load, 8)),
        "fsw" => Some((FloatKind::Store, 4)),
        "fsd" => Some((FloatKind::Store, 8)),
        _ => None,
    }
}

/// Returns if `name` is the name of a floating-point register.
#[must_use]
pub fn is_float_register(name: &str) -> bool {
    let number = |prefix: &str, max: u32| {
        name.strip_prefix(prefix)
            .and_then(|n| n.parse::<u32>().ok())
            .is_some_and(|n| n <= max && (n == 0 || !name[prefix.len()..].starts_with('0')))
    };
    number("f", 31) || number("ft", 11) || number("fs", 11) || number("fa", 7)
}

/// Returns if `name` is a static rounding mode operand.
#[must_use]
pub fn is_rounding_mode(name: &str) -> bool {
    matches!(name, "rne" | "rtz" | "rdn" | "rup" | "rmm" | "dyn")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_mnemonics() {
        assert_eq!(float_kind("fcvt.d.w"), Some(FloatKind::FromInt));
        assert_eq!(float_kind("fcvt.w.d"), Some(FloatKind::ToInt));
        assert_eq!(float_kind("fmv.x.w"), Some(FloatKind::ToInt));
        assert_eq!(float_kind("fmv.w.x"), Some(FloatKind::FromInt));
        assert_eq!(float_kind("flt.d"), Some(FloatKind::ToInt));
        assert_eq!(float_kind("fmadd.d"), Some(FloatKind::Compute));
        assert_eq!(float_kind("FADD.S"), Some(FloatKind::Compute));
        assert_eq!(float_kind("fcvt.s.d"), Some(FloatKind::Compute));
        assert_eq!(float_kind("fence.i"), None);
        assert_eq!(float_kind("fadd"), None);
        assert_eq!(float_memory_kind("fsd"), Some((FloatKind::Store, 8)));
        assert_eq!(float_memory_kind("flw"), Some((FloatKind::Load, 4)));
    }

    #[test]
    fn recognizes_registers() {
        for name in ["f0", "f31", "ft0", "ft11", "fs0", "fs11", "fa0", "fa7"] {
            assert!(is_float_register(name), "{name}");
        }
        for name in ["f32", "ft12", "fa8", "f01", "fp", "a0", "fs", "f"] {
            assert!(!is_float_register(name), "{name}");
        }
    }
}
