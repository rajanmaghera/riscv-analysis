use crate::analysis::AvailableValue;
use crate::cfg::Cfg;
use crate::parser::{InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, LintPassDefaultOptions};

// Check that we know the stack position at every point in the program (aka. within scopes)
#[non_exhaustive]
pub struct StackPass {
    default_options: LintPassDefaultOptions,
}
impl StackPass {
    pub fn new() -> Self {
        Self {
            default_options: LintPassDefaultOptions::default(),
        }
    }
}

impl Default for StackPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for StackPass {
    fn get_pass_name(&self) -> &'static str {
        "stack"
    }
    fn get_default_options(&self) -> &LintPassDefaultOptions {
        &self.default_options
    }

    fn get_default_options_mut(&mut self) -> &mut LintPassDefaultOptions {
        &mut self.default_options
    }
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
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
