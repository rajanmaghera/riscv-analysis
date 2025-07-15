use crate::cfg::Cfg;
use crate::parser::{HasRegisterSets, InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, PassConfiguration};

// check if the value of a calle-saved register is read as its original value
pub struct CalleeSavedGarbageReadCheck;
impl LintPass<CalleeSavedGarbageReadCheckConfiguration> for CalleeSavedGarbageReadCheck {
    fn run(cfg: &Cfg, errors: &mut DiagnosticManager, config: &CalleeSavedGarbageReadCheckConfiguration) {
        if !config.get_enabled() {
            return;
        }
        for node in cfg {
            for read in node.reads_from() {
                // if the node uses a calle saved register but not a memory access and the value going in is the original value, then we are reading a garbage value
                // DESIGN DECISION: we allow any memory accesses for calle saved registers

                if Register::saved_set().contains(read.get())
                    && node.uses_memory_location().is_none()
                    && node.reg_values_in().is_original_value(*read.get())
                {
                    errors.push(LintError::InvalidUseBeforeAssignment(read.clone()));
                    // then we are reading a garbage value
                }
            }
        }
    }
}
pub struct CalleeSavedGarbageReadCheckConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl Default for CalleeSavedGarbageReadCheckConfiguration {
    fn default() -> Self {
        CalleeSavedGarbageReadCheckConfiguration { enabled: true }
    }
}
impl PassConfiguration for CalleeSavedGarbageReadCheckConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}
// TODO check if the stack is ever stored at 0 or what not
