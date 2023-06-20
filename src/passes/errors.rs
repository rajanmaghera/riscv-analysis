use crate::parser::LabelString;

use crate::parser::Register;
use crate::parser::{Range, With};

#[derive(Debug)]
// Read/write within the text section
pub enum PassError {
    // if a loop variable does not change, then it will infinitely run
    // if a branch is always going to execute (i.e. if true) using constants and zero register
    InvalidUseAfterCall(Range, With<LabelString>),
    InvalidUseBeforeAssignment(Range),
    // TODO add tokens/registers to all
    // separate into invalid use
    OverwriteCalleeSavedRegister(Range, Register),
    ImproperFuncEntry(Range, With<LabelString>), // if a function has any prev items, (including program entry)
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
}

pub enum WarningLevel {
    Warning,
    Error,
}

impl From<&PassError> for WarningLevel {
    fn from(val: &PassError) -> Self {
        match val {
            PassError::DeadAssignment(_)
            | PassError::SaveToZero(_)
            | PassError::ImproperFuncEntry(..)
            | PassError::UnreachableCode(_) => WarningLevel::Warning,
            PassError::UnknownEcall(_)
            | PassError::InvalidUseAfterCall(_, _)
            | PassError::InvalidUseBeforeAssignment(_)
            | PassError::UnknownStack(_)
            | PassError::InvalidStackPointer(_)
            | PassError::InvalidStackPosition(_, _)
            | PassError::OverwriteCalleeSavedRegister(_, _) => WarningLevel::Error,
        }
    }
}

// implement display for passerror
impl std::fmt::Display for PassError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PassError::DeadAssignment(_) => write!(f, "Unused value"),
            PassError::SaveToZero(_) => write!(f, "Saving to zero register"),
            PassError::InvalidUseAfterCall(_, _) => write!(f, "Invalid use after call"),
            PassError::ImproperFuncEntry(..) => write!(f, "Improper function entry"),
            PassError::UnknownEcall(_) => write!(f, "Unknown ecall"),
            PassError::UnreachableCode(_) => write!(f, "Unreachable code"),
            PassError::InvalidUseBeforeAssignment(_) => write!(f, "Invalid use before assignment"),
            PassError::UnknownStack(_) => write!(f, "Unknown stack value"),
            PassError::InvalidStackPointer(_) => write!(f, "Invalid stack pointer"),
            PassError::InvalidStackPosition(_, _) => write!(f, "Invalid stack position"),
            PassError::OverwriteCalleeSavedRegister(_, _) => {
                write!(f, "Overwriting callee-saved register")
            }
        }
    }
}

impl PassError {
    pub fn long_description(&self) -> String {
        match self {
            PassError::DeadAssignment(_) => "Unused value".to_string(),
            PassError::SaveToZero(_) => "The result of this instruction is being stored to the zero (x0) register. This instruction has no effect.".to_string(),
            PassError::InvalidUseAfterCall(_,x) => format!("Register were read from after a function call to {}. Reading from these registers is invalid and likely contain garbage values.\n\nIt is possible that this register was not defined across every path within the function. If you expected this register to be a return value, re-examine the function definition.",
                x.data.0
        ),
            PassError::ImproperFuncEntry(..) => "This function can be entered through non-conventional ways. Either by the code before or through a jump. This label is treated like a function because there is either a [jal] instruction or an explicit definition of this function.".to_string(),
            PassError::UnknownEcall(_) => "The ecall type is not recognized. It is possible that you did not set a7 to a value.".to_string(),
            PassError::UnreachableCode(_) => "This code is unreachable. It is possible that you have a jump to a label that does not exist.".to_string(),
            PassError::InvalidUseBeforeAssignment(_) => "This register is being used before it is assigned to.".to_string(),
            PassError::UnknownStack(_) => "The stack value is not definitely known.".to_string(),
            PassError::InvalidStackPointer(_) => "The stack pointer is being overwritten.".to_string(),
            PassError::InvalidStackPosition(_, _) => "The stack value is wrong way (positive).".to_string(),
            PassError::OverwriteCalleeSavedRegister(_, x) => format!("Register {x} is being overwritten without the original value being restored at the end of the function. This register is callee-saved and should not be overwritten.
            You should be saving this register to the stack at the start of the function and restoring it at the end of the function."),
            // TODO extend Overwrite with real value analysis if known
            // You saved the value of xx to the stack on line xx. Perhaps you meant
            // to restore from this value instead.
        }
    }

    pub fn range(&self) -> Range {
        match self {
            PassError::DeadAssignment(r)
            | PassError::SaveToZero(r)
            | PassError::InvalidUseAfterCall(r, _)
            | PassError::InvalidUseBeforeAssignment(r)
            | PassError::ImproperFuncEntry(r, _)
            | PassError::UnknownEcall(r)
            | PassError::UnreachableCode(r)
            | PassError::UnknownStack(r)
            | PassError::InvalidStackPointer(r)
            | PassError::InvalidStackPosition(r, _)
            | PassError::OverwriteCalleeSavedRegister(r, _) => r.clone(),
        }
    }
}
