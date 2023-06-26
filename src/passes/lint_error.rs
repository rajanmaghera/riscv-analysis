use std::rc::Rc;

use crate::cfg::Function;

use crate::parser::Range;
use crate::parser::Register;

#[derive(Debug)]
// Read/write within the text section

// TODO REPLACE RANGES WITH With<___ like Register>
pub enum LintError {
    // if a loop variable does not change, then it will infinitely run
    // if a branch is always going to execute (i.e. if true) using constants and zero register
    InvalidUseAfterCall(Range, Rc<Function>),
    InvalidUseBeforeAssignment(Range),
    // TODO add tokens/registers to all
    // separate into invalid use
    OverwriteCalleeSavedRegister(Range, Register),
    ImproperFuncEntry(Range, Rc<Function>), // if a function has any prev items, (including program entry)
    DeadAssignment(Range),
    SaveToZero(Range),
    UnknownEcall(Range),
    UnknownStack(Range),              // stack value is not definitely known
    InvalidStackPointer(Range),       // stack value is being overwritten
    InvalidStackPosition(Range, i32), // stack value is wrong way (positive)
    UnreachableCode(Range),           // -- code that is unreachable
                                      // SetBadRegister(Range, Register), -- used when setting registers that should not be set
                                      // OverwriteRaRegister(Range), -- used when overwriting the return address register
                                      // OverwriteRegister(Range, Register), -- used when overwriting a register that has not been saved
                                      // FallOffEnd(Range), program may fall off the end of code
                                      // InvalidControlFlowRead(Range), -- reading from a register that is not assigned to
                                      // ProgramExit in the middle of a function
                                      // NonMatchingOffset -- if the multiple of the offset does not match the instruction (ex. 4 for lw), then it is a warning
}

pub enum WarningLevel {
    Warning,
    Error,
}

impl From<&LintError> for WarningLevel {
    fn from(val: &LintError) -> Self {
        match val {
            LintError::DeadAssignment(_)
            | LintError::SaveToZero(_)
            | LintError::ImproperFuncEntry(..)
            | LintError::UnreachableCode(_) => WarningLevel::Warning,
            LintError::UnknownEcall(_)
            | LintError::InvalidUseAfterCall(_, _)
            | LintError::InvalidUseBeforeAssignment(_)
            | LintError::UnknownStack(_)
            | LintError::InvalidStackPointer(_)
            | LintError::InvalidStackPosition(_, _)
            | LintError::OverwriteCalleeSavedRegister(_, _) => WarningLevel::Error,
        }
    }
}

// implement display for passerror
impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LintError::DeadAssignment(_) => write!(f, "Unused value"),
            LintError::SaveToZero(_) => write!(f, "Saving to zero register"),
            LintError::InvalidUseAfterCall(_, _) => write!(f, "Invalid use after call"),
            LintError::ImproperFuncEntry(..) => write!(f, "Improper function entry"),
            LintError::UnknownEcall(_) => write!(f, "Unknown ecall"),
            LintError::UnreachableCode(_) => write!(f, "Unreachable code"),
            LintError::InvalidUseBeforeAssignment(_) => write!(f, "Invalid use before assignment"),
            LintError::UnknownStack(_) => write!(f, "Unknown stack value"),
            LintError::InvalidStackPointer(_) => write!(f, "Invalid stack pointer"),
            LintError::InvalidStackPosition(_, _) => write!(f, "Invalid stack position"),
            LintError::OverwriteCalleeSavedRegister(_, _) => {
                write!(f, "Overwriting callee-saved register")
            }
        }
    }
}

impl LintError {
    pub fn long_description(&self) -> String {
        match self {
            LintError::DeadAssignment(_) => "Unused value".to_string(),
            LintError::SaveToZero(_) => "The result of this instruction is being stored to the zero (x0) register. This instruction has no effect.".to_string(),
            LintError::InvalidUseAfterCall(_,x) => format!("Register were read from after a function call to {}. Reading from these registers is invalid and likely contain garbage values.\n\nIt is possible that this register was not defined across every path within the function. If you expected this register to be a return value, re-examine the function definition.",
                x.entry.labels.iter().map(|x| x.data.0.clone()).collect::<Vec<_>>().join(", ")
        ),
            LintError::ImproperFuncEntry(..) => "This function can be entered through non-conventional ways. Either by the code before or through a jump. This label is treated like a function because there is either a [jal] instruction or an explicit definition of this function.".to_string(),
            LintError::UnknownEcall(_) => "The ecall type is not recognized. It is possible that you did not set a7 to a value.".to_string(),
            LintError::UnreachableCode(_) => "This code is unreachable. It is possible that you have a jump to a label that does not exist.".to_string(),
            LintError::InvalidUseBeforeAssignment(_) => "This register is being used before it is assigned to.".to_string(),
            LintError::UnknownStack(_) => "The stack value is not definitely known.".to_string(),
            LintError::InvalidStackPointer(_) => "The stack pointer is being overwritten.".to_string(),
            LintError::InvalidStackPosition(_, _) => "The stack value is wrong way (positive).".to_string(),
            LintError::OverwriteCalleeSavedRegister(_, x) => format!("Register {x} is being overwritten without the original value being restored at the end of the function. This register is callee-saved and should not be overwritten.
            You should be saving this register to the stack at the start of the function and restoring it at the end of the function."),
            // TODO extend Overwrite with real value analysis if known
            // You saved the value of xx to the stack on line xx. Perhaps you meant
            // to restore from this value instead.
        }
    }

    pub fn range(&self) -> Range {
        match self {
            LintError::DeadAssignment(r)
            | LintError::SaveToZero(r)
            | LintError::InvalidUseAfterCall(r, _)
            | LintError::InvalidUseBeforeAssignment(r)
            | LintError::ImproperFuncEntry(r, _)
            | LintError::UnknownEcall(r)
            | LintError::UnreachableCode(r)
            | LintError::UnknownStack(r)
            | LintError::InvalidStackPointer(r)
            | LintError::InvalidStackPosition(r, _)
            | LintError::OverwriteCalleeSavedRegister(r, _) => r.clone(),
        }
    }
}
