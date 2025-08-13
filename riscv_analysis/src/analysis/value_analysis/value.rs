use crate::parser::{RVRegister, RegisterProperties};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Value {
    Unknown,
    UnknownConst,
    /// Stack pointer with offset, equivalent to Initial(sp) + i32
    InitialStackPointer(i32),
    Initial(RVRegister),
    Const(i32),
    Undefined,
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unknown => write!(f, "T"),
            Value::UnknownConst => write!(f, "Tc"),
            Value::InitialStackPointer(x) => {
                write!(f, "init({}) + {}", RVRegister::stack_pointer(), x)
            }
            Value::Initial(reg) => write!(f, "init({reg})"),
            Value::Const(x) => write!(f, "{x}"),
            Value::Undefined => write!(f, "B"),
        }
    }
}

impl Value {
    /// Perform a LUB operation.
    #[must_use] pub fn join(self, other: Value) -> Value {
        match (self.canonicalize(), other.canonicalize()) {
            (x, y) if x == y => y,
            (Value::Undefined, x) | (x, Value::Undefined) => x,
            (Value::Unknown | Value::InitialStackPointer(_), _) |
(_, Value::Unknown | Value::InitialStackPointer(_)) => Value::Unknown,
            (_, _) => Value::UnknownConst,
        }
    }

    #[must_use] pub fn canonicalize(self) -> Self {
        match self {
            Value::Initial(reg) if reg.is_stack_pointer() => Value::InitialStackPointer(0),
            _ => self,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn equal_values_return_same() {
        let r1 = RVRegister::X1;
        assert_eq!(Value::Const(42).join(Value::Const(42)), Value::Const(42));
        assert_eq!(
            Value::InitialStackPointer(4).join(Value::InitialStackPointer(4)),
            Value::InitialStackPointer(4)
        );
        assert_eq!(
            Value::Initial(r1).join(Value::Initial(r1)),
            Value::Initial(r1)
        );
    }

    #[test]
    fn undefined_is_neutral_element() {
        assert_eq!(Value::Undefined.join(Value::Const(10)), Value::Const(10));
        assert_eq!(
            Value::InitialStackPointer(1).join(Value::Undefined),
            Value::InitialStackPointer(1)
        );
    }

    #[test]
    fn unknown_is_top_element() {
        let r1 = RVRegister::X1;
        assert_eq!(Value::Unknown.join(Value::Const(99)), Value::Unknown);
        assert_eq!(Value::Initial(r1).join(Value::Unknown), Value::Unknown);
    }

    #[test]
    fn stackpointer_always_results_in_unknown_if_not_equal() {
        let r2 = RVRegister::X10;
        assert_eq!(
            Value::InitialStackPointer(1).join(Value::Const(5)),
            Value::Unknown
        );
        assert_eq!(
            Value::Initial(r2).join(Value::InitialStackPointer(2)),
            Value::Unknown
        );
    }

    #[test]
    fn non_equal_non_special_values_result_in_unknown_const() {
        let r1 = RVRegister::X1;
        let r2 = RVRegister::X10;
        assert_eq!(Value::Const(10).join(Value::Const(20)), Value::UnknownConst);
        assert_eq!(
            Value::Initial(r1).join(Value::Initial(r2)),
            Value::UnknownConst
        );
        assert_eq!(
            Value::Initial(r1).join(Value::Const(20)),
            Value::UnknownConst
        );
        assert_eq!(
            Value::Const(10).join(Value::Initial(r2)),
            Value::UnknownConst
        );
    }
}
