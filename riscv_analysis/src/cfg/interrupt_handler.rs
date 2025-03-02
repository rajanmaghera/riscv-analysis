use std::collections::HashSet;

use crate::analysis::AvailableValue;
use crate::cfg::{Cfg, CfgNode};
use crate::parser::{CsrIType, CsrImm, CsrType, Imm, LabelStringToken, ParserNode, Register};

impl CsrImm {
    /// Returns if this CSR register is the interrupt vector (utvec).
    fn is_interrupt_vector(&self) -> bool {
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
    Register(Register),
}

impl CsrInstSource {
    /// Create a new CsrInstSource from an immediate value.
    fn from_immediate(value: Imm) -> Self {
        CsrInstSource::Imm(value)
    }

    /// Create a new CsrInstSource from a register value.
    fn from_register(value: Register) -> Self {
        CsrInstSource::Register(value)
    }
}

impl ParserNode {
    /// Returns if this node sets a CSR register and what it is set to.
    ///
    /// This function ignores any other operations other than csrrw and csrrwi. The logical OR and
    /// clear bits functionality is not covered.
    fn sets_csr(&self) -> Option<(CsrImm, CsrInstSource)> {
        match self {
            ParserNode::Csr(node) if matches!(node.inst.get(), CsrType::Csrrw) => Some((
                node.csr.get_cloned(),
                CsrInstSource::from_register(*node.rs1.get()),
            )),
            ParserNode::CsrI(node) if matches!(node.inst.get(), CsrIType::Csrrwi) => Some((
                node.csr.get_cloned(),
                CsrInstSource::from_immediate(node.imm.get_cloned()),
            )),
            _ => None,
        }
    }
}

impl CfgNode {
    /// Returns the number of a CSR register that is set by this node and what it is set to.
    ///
    /// It is possible that a node sets a value, but it is not known what the value is. In that case, we
    /// will return None in the location of the AvailableValue. CSR instructions that use
    /// immediates are always known and converted to their constant AvailableValue counterparts.
    fn sets_csr_to_value(&self) -> Option<(CsrImm, Option<AvailableValue>)> {
        if let Some((csr, value)) = self.node().sets_csr() {
            match value {
                CsrInstSource::Imm(imm) => Some((csr, Some(AvailableValue::Constant(imm.value())))),
                CsrInstSource::Register(reg) => {
                    Some((csr, self.reg_values_in().get(&reg).cloned()))
                }
            }
        } else {
            None
        }
    }
}

impl Cfg {
    /// On a CFG with NodeDirections and AvailableValues, get the names of the interrupt handler functions.
    ///
    /// This function looks for labels that are set to the CSR utvec (interrupt vector) and returns them.
    /// It requires that there are directions and available values in the CFG.
    ///
    /// The intended use of this function is to find the names, then regenerate the CFG with the interrupt
    /// handler names as predefined call names.
    pub fn get_names_of_interrupt_handler_functions(&self) -> HashSet<LabelStringToken> {
        let mut interrupt_handler_names = HashSet::new();

        // Look for names of labels that are set to the interrupt vector CSR.
        for node in self {
            if let Some((csr, Some(AvailableValue::Address(label)))) = node.sets_csr_to_value() {
                if csr.is_interrupt_vector() {
                    interrupt_handler_names.insert(label);
                }
            }
        }

        interrupt_handler_names
    }
}
