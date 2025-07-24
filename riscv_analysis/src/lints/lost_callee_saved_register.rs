use crate::analysis::AvailableValue;
use crate::cfg::Cfg;
use crate::parser::{HasRegisterSets, InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, LintPassDefaultOptions};

// Check if the value of a calle-saved register is ever "lost" (aka. overwritten without being restored)
// This provides a more detailed image compared to above, and could be turned into extra
// diagnostic information in the future.
#[non_exhaustive]
pub struct LostCalleeSavedRegisterPass {
    default_options: LintPassDefaultOptions,
}
impl LostCalleeSavedRegisterPass {
    pub fn new() -> Self {
        Self {
            default_options: LintPassDefaultOptions::default(),
        }
    }
}

impl Default for LostCalleeSavedRegisterPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for LostCalleeSavedRegisterPass {
    fn get_pass_name(&self) -> &'static str {
        "lost-called-saved-register"
    }
    fn get_default_options(&self) -> &LintPassDefaultOptions {
        &self.default_options
    }

    fn get_default_options_mut(&mut self) -> &mut LintPassDefaultOptions {
        &mut self.default_options
    }
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg {
            let callee = Register::saved_set();

            // If: within a function, node stores to a saved register,
            // and the value going in was the original value
            // We intentionally do not check for callee-saved registers
            // as the value is mean to be modified
            if let Some(reg) = node.writes_to() {
                if callee.contains(reg.get())
                    && node.is_part_of_some_function()
                    && node.reg_values_in().get(reg.get())
                        == Some(&AvailableValue::OriginalRegisterWithScalar(*reg.get(), 0))
                {
                    // Check that the value exists somewhere in the available values
                    // like the stack.
                    // if not, then we have a problem
                    let mut found = false;

                    // check stack values:
                    let stack = node.memory_values_out();
                    for (_, val) in stack {
                        if let AvailableValue::OriginalRegisterWithScalar(reg2, offset) = val {
                            if reg2 == *reg.get() && offset == 0 {
                                found = true;
                                break;
                            }
                        }
                    }

                    // check register values:
                    let regs = node.reg_values_out();
                    for (_, val) in regs {
                        if let AvailableValue::OriginalRegisterWithScalar(reg2, offset) = val {
                            if reg2 == *reg.get() && offset == 0 {
                                found = true;
                                break;
                            }
                        }
                    }

                    if !found {
                        errors.push(LintError::LostRegisterValue(reg));
                    }
                }
            }
        }
    }
}
