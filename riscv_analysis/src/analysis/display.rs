use super::AvailableValue;

impl std::fmt::Display for AvailableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvailableValue::MemoryAtCsr(csr, offset) => write!(f, "{offset}(csr[{}])", csr.0),
            AvailableValue::ValueInCsr(csr) => write!(f, "csr[{}]", csr.0),
            AvailableValue::Constant(v) => write!(f, "{v}"),
            AvailableValue::Address(a) => write!(f, "{a}"),
            AvailableValue::Memory(a, off) => write!(f, "{off}({a})"),
            AvailableValue::OriginalRegisterWithScalar(reg, off) => {
                if off == &0 {
                    write!(f, "{reg}")
                } else {
                    write!(f, "{reg} + {off}")
                }
            }
            AvailableValue::RegisterWithScalar(reg, off) => {
                if off == &0 {
                    write!(f, "{reg}?")
                } else {
                    write!(f, "{reg}? + {off}")
                }
            }
            AvailableValue::MemoryAtRegister(reg, off) => write!(f, "{off}({reg}?)"),
            AvailableValue::MemoryAtOriginalRegister(reg, off) => {
                write!(f, "{off}({reg})")
            }
        }
    }
}
