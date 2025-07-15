use crate::cfg::Cfg;
use crate::parser::{HasRegisterSets, InstructionProperties, Register};
use crate::passes::{DiagnosticManager, LintError, LintPass, PassConfiguration};

// check if the value of a callee-saved register is read as its original value
pub struct CalleeSavedGarbageReadPass;
impl LintPass<CalleeSavedGarbageReadPassConfiguration> for CalleeSavedGarbageReadPass {
    fn run(
        cfg: &Cfg,
        errors: &mut DiagnosticManager,
        config: &CalleeSavedGarbageReadPassConfiguration,
    ) {
        if !config.get_enabled() {
            return;
        }
        for node in cfg {
            for read in node.reads_from() {
                // if the node uses a callee saved register but not a memory access and the value going in is the original value, then we are reading a garbage value
                // DESIGN DECISION: we allow any memory accesses for callee saved registers

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
pub struct CalleeSavedGarbageReadPassConfiguration {
    /// Is the pass enabled?
    enabled: bool,
}
impl Default for CalleeSavedGarbageReadPassConfiguration {
    fn default() -> Self {
        CalleeSavedGarbageReadPassConfiguration { enabled: true }
    }
}
impl PassConfiguration for CalleeSavedGarbageReadPassConfiguration {
    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
// TODO check if the stack is ever stored at 0 or what not
