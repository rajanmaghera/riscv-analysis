use crate::analysis::AvailableValue;
use crate::cfg::Cfg;
use crate::parser::{HasRegisterSets, InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass};

// Check if the value of a calle-saved register is ever "lost" (aka. overwritten without being restored)
// This provides a more detailed image compared to above, and could be turned into extra
// diagnostic information in the future.
pub struct LostCalleeSavedRegisterCheck;

impl LintPass for LostCalleeSavedRegisterCheck {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager) {
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
