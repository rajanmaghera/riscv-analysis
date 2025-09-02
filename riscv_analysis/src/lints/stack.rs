use crate::analysis::{Location, Value};
use crate::cfg::{Cfg, Segment};
use crate::parser::{InstructionProperties, RVRegister};
use crate::passes::{DiagnosticBuilder, DiagnosticCertainty, DiagnosticManager, LintPass};

// Check that we know the stack position at every point in the program (aka. within scopes)
#[non_exhaustive]
pub struct StackPass;
impl StackPass {
    #[must_use]
    pub fn new() -> Self {
        Self {}
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
    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        for node in cfg.iter_source() {
            if node.segment() != Segment::Text {
                continue;
            }
            if node.calls_to().is_some() {
                // At each function call instruction,
                // check that the stack pointer is a negative multiple of 4
                match node
                    .real_val_in()
                    .get(&Location::Register(RVRegister::stack_pointer()))
                {
                    Value::InitialStackPointer(x) if x % 4 == 0 && x <= 0 => {
                        // Good, this program follows conventions
                    }
                    Value::InitialStackPointer(x) if x > 0 => {
                        // Caller stack frame will be corrupted
                        errors.push(
                            DiagnosticBuilder::new(
                                "caller-stack-frame-corruption",
                                "Caller stack frame will be corrupted",
                            )
                            .with_certainty(DiagnosticCertainty::IsTrueIfAssumptionsAreMet)
                            .is_error_on(node.as_ref()),
                        );
                    }
                    Value::InitialStackPointer(x) if x % 4 != 0 => errors.push(
                        DiagnosticBuilder::new(
                            "stack-pointer-misalignment",
                            "Stack pointer is misaligned",
                        )
                        .with_certainty(DiagnosticCertainty::IsTrueIfAssumptionsAreMet)
                        .is_error_on(node.as_ref()),
                    ),
                    Value::Unknown => errors.push(
                        DiagnosticBuilder::new(
                            "stack-pointer-maybe-invalid",
                            "Stack pointer may be incorrect",
                        )
                        .with_certainty(DiagnosticCertainty::MightBeTrue)
                        .is_warning_on(node.as_ref()),
                    ),
                    x => errors.push(
                        DiagnosticBuilder::new("stack-pointer-invalid", "Stack pointer is invalid")
                            .description(format!(
                                "The stack pointer should be sp - x (some constant), but it is {x}"
                            ))
                            .with_certainty(DiagnosticCertainty::IsTrueIfAssumptionsAreMet)
                            .is_error_on(node.as_ref()),
                    ),
                }
            }
        }
    }
}
