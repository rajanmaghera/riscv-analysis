

use crate::parser::ast::LabelString;

use crate::parser::register::Register;
use crate::parser::token::{Range, WithToken};

#[derive(Debug)]
pub struct PassErrors {
    pub errors: Vec<PassError>,
}

#[derive(Debug)]
// Read/write within the text section
pub enum PassError {
    // if a loop variable does not change, then it will infinitely run
    // if a branch is always going to execute (i.e. if true) using constants and zero register
    InvalidUseAfterCall(Range, WithToken<LabelString>),
    InvalidUseBeforeAssignment(Range),
    // TODO add tokens/registers to all
    // separate into invalid use
    OverwriteCalleeSavedRegister(Range, Register),
    ImproperFuncEntry(Range, WithToken<LabelString>), // if a function has any prev items, (including program entry)
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
    Suggestion,
    Warning,
    Error,
}

impl Into<WarningLevel> for &PassError {
    fn into(self) -> WarningLevel {
        match self {
            PassError::DeadAssignment(_) => WarningLevel::Warning,
            PassError::SaveToZero(_) => WarningLevel::Warning,
            PassError::InvalidUseAfterCall(_, _) => WarningLevel::Error,
            PassError::ImproperFuncEntry(..) => WarningLevel::Warning,
            PassError::UnknownEcall(_) => WarningLevel::Error,
            PassError::UnreachableCode(_) => WarningLevel::Warning,
            PassError::InvalidUseBeforeAssignment(_) => WarningLevel::Error,
            PassError::UnknownStack(_) => WarningLevel::Error,
            PassError::InvalidStackPointer(_) => WarningLevel::Error,
            PassError::InvalidStackPosition(_, _) => WarningLevel::Error,
            PassError::OverwriteCalleeSavedRegister(_, _) => WarningLevel::Error,
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
        ).to_string(),
            PassError::ImproperFuncEntry(..) => "This function can be entered through non-conventional ways. Either by the code before or through a jump. This label is treated like a function because there is either a [jal] instruction or an explicit definition of this function.".to_string(),
            PassError::UnknownEcall(_) => "The ecall type is not recognized. It is possible that you did not set a7 to a value.".to_string(),
            PassError::UnreachableCode(_) => "This code is unreachable. It is possible that you have a jump to a label that does not exist.".to_string(),
            PassError::InvalidUseBeforeAssignment(_) => "This register is being used before it is assigned to.".to_string(),
            PassError::UnknownStack(_) => "The stack value is not definitely known.".to_string(),
            PassError::InvalidStackPointer(_) => "The stack pointer is being overwritten.".to_string(),
            PassError::InvalidStackPosition(_, _) => "The stack value is wrong way (positive).".to_string(),
            PassError::OverwriteCalleeSavedRegister(_, x) => format!("Register {} is being overwritten without the original value being restored at the end of the function. This register is callee-saved and should not be overwritten.
            You should be saving this register to the stack at the start of the function and restoring it at the end of the function.", x).to_string(),
            // TODO extend Overwrite with real value analysis if known
            // You saved the value of xx to the stack on line xx. Perhaps you meant
            // to restore from this value instead.
        }
    }

    pub fn range(&self) -> Range {
        match self {
            PassError::DeadAssignment(range) => range.clone(),
            PassError::SaveToZero(range) => range.clone(),
            PassError::InvalidUseAfterCall(range, _) => range.clone(),
            PassError::InvalidUseBeforeAssignment(range) => range.clone(),
            PassError::ImproperFuncEntry(range, _) => range.clone(),
            PassError::UnknownEcall(range) => range.clone(),
            PassError::UnreachableCode(range) => range.clone(),
            PassError::UnknownStack(range) => range.clone(),
            PassError::InvalidStackPointer(range) => range.clone(),
            PassError::InvalidStackPosition(range, _) => range.clone(),
            PassError::OverwriteCalleeSavedRegister(range, _) => range.clone(),
        }
    }
}
