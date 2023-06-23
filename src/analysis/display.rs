impl Display for AvailableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AvailableValue::Constant(v) => write!(f, "{v}"),
            AvailableValue::MemAddr(a) => write!(f, "{a}"),
            AvailableValue::Memory(a, off) => write!(f, "{off}({a})"),
            AvailableValue::CurrMemReg(reg, off) => write!(f, "{off}(<{reg}>?)"),
            AvailableValue::CurrScalarOffset(reg, off) => {
                if off == &0 {
                    write!(f, "<{reg}>?")
                } else {
                    write!(f, "<{reg}>? + {off}")
                }
            }
            AvailableValue::OrigScalarOffset(reg, off) => {
                if off == &0 {
                    write!(f, "{reg}")
                } else {
                    write!(f, "{reg} + {off}")
                }
            }
            AvailableValue::OrigMemReg(reg, off) => write!(f, "{off}({reg})"),
        }
    }
}
