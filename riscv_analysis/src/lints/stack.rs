use crate::analysis::AvailableValue;
use crate::cfg::Cfg;
use crate::parser::{InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, PassConfiguration};

// Check that we know the stack position at every point in the program (aka. within scopes)
pub struct StackPass;
impl LintPass<StackPassConfiguration> for StackPass {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &StackPassConfiguration) {
        if !config.get_enabled() {
            return;
        }
        // PASS 1
        // check that we know the stack position at every point in the program
        // check that the stack is never in an invalid position
        // TODO check that the stack stores always happen to a place that is negative
        // TODO move to impl methods
        'outer: for node in cfg {
            let values = node.reg_values_out();
            match values.get(&Register::X2) {
                None => {
                    errors.push(LintError::UnknownStack(node.node()));
                    break 'outer;
                }
                Some(x) => {
                    if let AvailableValue::OriginalRegisterWithScalar(reg, off) = x {
                        if reg != &Register::X2 {
                            errors.push(LintError::InvalidStackPointer(node.node()));
                            break 'outer;
                        }
                        if off > &0 {
                            errors.push(LintError::InvalidStackPosition(node.node(), *off));
                            break 'outer;
                        }

                        if let Some((reg2, off2)) = node.uses_memory_location() {
                            if reg2 == Register::X2 && off2.value() + off >= 0 {
                                errors.push(LintError::InvalidStackOffsetUsage(
                                    node.node().clone(),
                                    off2.value() + off,
                                ));
                            }
                        }
                    } else {
                        errors.push(LintError::InvalidStackPointer(node.node()));
                        break 'outer;
                    }
                }
            }
            if let Some((reg, _)) = node.uses_memory_location() {
                if reg == Register::X2 {}
            }
        }

        // PASS 2: check that
    }
}
pub struct StackPassConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl Default for StackPassConfiguration {
    fn default() -> Self {
        StackPassConfiguration { enabled: true }
    }
}
impl PassConfiguration for StackPassConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
