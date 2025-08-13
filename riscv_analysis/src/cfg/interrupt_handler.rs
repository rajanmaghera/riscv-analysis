
use crate::parser::{
    CsrIType, CsrImm, CsrType, Imm, RVInstructionNode, RVRegister,
};

impl CsrImm {
    /// Returns if this CSR register is the interrupt vector (utvec).
    fn is_interrupt_vector(self) -> bool {
        self.value() == 0x005
    }
}

/// The source value of a CSR instruction.
///
/// This can be either an immediate value or a value in a register.
enum CsrInstSource {
    /// An immediate value.
    Imm(Imm),
    /// A value in a register.
    Register(RVRegister),
}

impl CsrInstSource {
    /// Create a new `CsrInstSource` from an immediate value.
    fn from_immediate(value: Imm) -> Self {
        CsrInstSource::Imm(value)
    }

    /// Create a new `CsrInstSource` from a register value.
    fn from_register(value: RVRegister) -> Self {
        CsrInstSource::Register(value)
    }
}

impl RVInstructionNode {
    /// Returns if this node sets a CSR register and what it is set to.
    ///
    /// This function ignores any other operations other than csrrw and csrrwi. The logical OR and
    /// clear bits functionality is not covered.
    fn sets_csr(&self) -> Option<(CsrImm, CsrInstSource)> {
        match self {
            RVInstructionNode::Csr(node) if matches!(node.inst.get(), CsrType::Csrrw) => Some((
                node.csr.get_cloned(),
                CsrInstSource::from_register(*node.rs1.get()),
            )),
            RVInstructionNode::CsrI(node) if matches!(node.inst.get(), CsrIType::Csrrwi) => Some((
                node.csr.get_cloned(),
                CsrInstSource::from_immediate(node.imm.get_cloned()),
            )),
            _ => None,
        }
    }
}
