use std::rc::Rc;

use crate::{
    cfg::Cfg,
    parser::{HasRegisterSets, InstructionProperties, Register},
    passes::{DiagnosticManager, LintError, LintPass},
};

#[non_exhaustive]
pub struct DeadValuePass;
impl DeadValuePass {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DeadValuePass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for DeadValuePass {
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg {
            // check the out of the node for any uses that
            // should not be there (temporaries)
            // TODO merge with Callee saved register check
            if let Some((function, call_site)) = node.calls_to_from_cfg(cfg) {
                // check the expected return values of the function:

                let out = (Register::caller_saved_set() - function.returns()) & node.live_out();

                // if there is anything left, then there is an error
                // for each item, keep going to the next node until a use of
                // that item is found
                let mut ranges = Vec::new();
                for item in &out {
                    ranges.append(&mut Cfg::error_ranges_for_first_usage(&node, item));
                }
                for item in ranges {
                    errors.push(LintError::InvalidUseAfterCall(
                        item,
                        Rc::clone(&function),
                        call_site.clone(),
                    ));
                }
            }
            // Check for any assignments that don't make it
            // to the end of the node. These assignments are not
            // used.
            else if let Some(def) = node.writes_to() {
                if !node.live_out().contains(def.get()) && !node.can_skip_save_checks() {
                    errors.push(LintError::DeadAssignment(def));
                }
            }
        }
    }
}
