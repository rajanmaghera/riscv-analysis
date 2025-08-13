use crate::cfg::Cfg;
use crate::passes::{DiagnosticManager, LintPass};

// check if the value of a calle-saved register is read as its original value
#[non_exhaustive]
pub struct CalleeSavedGarbageReadPass;
impl CalleeSavedGarbageReadPass {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CalleeSavedGarbageReadPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for CalleeSavedGarbageReadPass {
    fn get_pass_name(&self) -> &'static str {
        "callee-saved-garbage-read"
    }

    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        // for node in cfg.iter_source() {
        //     for read in node.reads_from() {
        //         // if the node uses a calle saved register but not a memory access and the value going in is the original value, then we are reading a garbage value
        //         // DESIGN DECISION: we allow any memory accesses for calle saved registers
        //
        //         if Register::saved_set().contains(read.get())
        //             && node.uses_memory_location().is_none()
        //             && node.reg_values_in().is_original_value(*read.get())
        //         {
        //             errors.push(LintError::InvalidUseBeforeAssignment(read.clone()));
        //             // then we are reading a garbage value
        //         }
        //     }
        // }
    }
}
