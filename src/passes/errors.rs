use crate::parser::ast::LabelString;

use crate::parser::token::{Range, WithToken};

#[derive(Debug)]
pub struct PassErrors {
    pub errors: Vec<PassError>,
}

#[derive(Debug)]
pub enum PassError {
    InvalidUseAfterCall(Range, WithToken<LabelString>),
    ImproperFuncEntry(Range, WithToken<LabelString>), // if a function has any prev items, (including program entry)
    DeadAssignment(Range),
    SaveToZero(Range),
    UnknownEcall(Range),
    UnreachableCode(Range), // -- code that is unreachable
                            // SetBadRegister(Range, Register), -- used when setting registers that should not be set
                            // OverwriteRaRegister(Range), -- used when overwriting the return address register
                            // OverwriteRegister(Range, Register), -- used when overwriting a register that has not been saved
                            // FallOffEnd(Range), program may fall off the end of code
                            // EcallNonLiveArgument -- ecall where expected argument based on a7 is not live
                            // \_ for this, use same logic as argument passing
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
        }
    }

    pub fn range(&self) -> Range {
        match self {
            PassError::DeadAssignment(range) => range.clone(),
            PassError::SaveToZero(range) => range.clone(),
            PassError::InvalidUseAfterCall(range, _) => range.clone(),
            PassError::ImproperFuncEntry(range, _) => range.clone(),
            PassError::UnknownEcall(range) => range.clone(),
            PassError::UnreachableCode(range) => range.clone(),
        }
    }
}