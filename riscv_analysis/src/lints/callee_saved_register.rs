use crate::analysis::{Location, Value};
use crate::cfg::Cfg;
use crate::parser::{HasRegisterSets, RVRegister, RegisterProperties};
use crate::passes::{DiagnosticBuilder, DiagnosticCertainty, DiagnosticManager, LintPass};
use itertools::{intersperse, Itertools};
use std::collections::HashSet;

// Check if the values of callee-saved registers are restored to the original value at the end of the function
#[non_exhaustive]
pub struct CalleeSavedRegisterPass;
impl CalleeSavedRegisterPass {
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CalleeSavedRegisterPass {
    fn default() -> Self {
        Self::new()
    }
}

impl LintPass for CalleeSavedRegisterPass {
    fn get_pass_name(&self) -> &'static str {
        "callee-saved-register"
    }

    fn run(&self, cfg: &Cfg, errors: &mut DiagnosticManager) {
        let registers_to_check = RVRegister::callee_saved_set();
        for func in cfg.get_all_functions() {
            for exit in func.exits().iter() {
                let mut invalid_registers = HashSet::new();
                let mut maybe_invalid_registers = HashSet::new();
                let exit_vals = exit.real_val_in();
                for reg in &registers_to_check {
                    match exit_vals.get(&Location::Register(reg)) {
                        Value::Initial(r) if !reg.is_stack_pointer() && r == reg => {
                            // Registers match the expected
                        }
                        Value::InitialStackPointer(x) if reg.is_stack_pointer() && x == 0 => {
                            // Register matche the expected
                        }
                        Value::UnknownConst | Value::Unknown => {
                            // Registers are a less precise version of what we expected
                            maybe_invalid_registers.insert(reg);
                        }
                        _ => {
                            //
                            invalid_registers.insert(reg);
                        }
                    }
                }
                if !invalid_registers.is_empty() {
                    errors.push(
                        DiagnosticBuilder::new(
                            "callee-saved-register-not-restored",
                            "Callee saved register is not restored",
                        )
                        .description(format!(
                            "Registers {} is/are not restored",
                            itertools::Itertools::intersperse(
                                invalid_registers.iter().sorted().map(std::string::ToString::to_string),
                                ", ".to_string()
                            )
                            .collect::<String>(),
                        ))
                        .with_certainty(DiagnosticCertainty::IsTrueIfAssumptionsAreMet)
                        .is_error_on(exit.as_ref()),
                    );
                }
                if !maybe_invalid_registers.is_empty() {
                    errors.push(
                        DiagnosticBuilder::new(
                            "callee-saved-register-maybe-not-restored",
                            "Callee saved register is maybe not restored",
                        )
                        .description(format!(
                            "Registers {} is/are potentially not restored",
                            intersperse(
                                maybe_invalid_registers
                                    .iter()
                                    .sorted()
                                    .map(std::string::ToString::to_string),
                                ", ".to_string()
                            )
                            .collect::<String>(),
                        ))
                        .with_certainty(DiagnosticCertainty::MightBeTrue)
                        .is_warning_on(exit.as_ref()),
                    );
                }
            }
        }
    }
}
