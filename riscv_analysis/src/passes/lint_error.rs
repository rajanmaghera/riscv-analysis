use std::rc::Rc;

use uuid::Uuid;

use crate::cfg::Function;

use crate::parser::LabelString;
use crate::parser::ParserNode;
use crate::parser::Range;
use crate::parser::Register;
use crate::parser::With;

use itertools::Itertools;

use super::DiagnosticLocation;
use super::DiagnosticMessage;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum LintError {
    // if a loop variable does not change, then it will infinitely run
    // if a branch is always going to execute (i.e. if true) using constants and zero register
    LostRegisterValue(With<Register>),

    /// A register 0 is used after a call to function 1 at call site 2
    InvalidUseAfterCall(With<Register>, Rc<Function>, With<LabelString>),
    InvalidUseBeforeAssignment(With<Register>),
    OverwriteCalleeSavedRegister(With<Register>),
    FirstInstructionIsFunction(ParserNode, Rc<Function>), // if the first instruction has a function, it is incorrect
    /// A function is entered through a non-conventional way
    ///
    /// If a function has any previous items, it is entered either through the
    /// instruction right before or a (un)conditional jump that is not a function
    /// call.
    ///
    /// (First line in function, line where function is entered through, function)
    InvalidJumpToFunction(ParserNode, ParserNode, Rc<Function>),
    DeadAssignment(With<Register>),
    SaveToZero(With<Register>),
    InvalidSegment(ParserNode),
    UnknownEcall(ParserNode),
    UnknownStack(ParserNode),        // stack value is not definitely known
    InvalidStackPointer(ParserNode), // stack value is being overwritten
    InvalidStackPosition(ParserNode, i32), // stack value is wrong way (positive)
    InvalidStackOffsetUsage(ParserNode, i32), // read/write using invalid stack offser
    UnreachableCode(ParserNode),     // -- code that is unreachable
    // SetBadRegister(Range, Register), -- used when setting registers that should not be set
    // FallOffEnd(Range), program may fall off the end of code
    // InvalidControlFlowRead(Range), -- reading from a register that is not assigned to
    // ProgramExit in the middle of a function
    // NonMatchingOffset -- if the multiple of the offset does not match the instruction (ex. 4 for lw), then it is a warning
    // LoadAddressFromTextLabel -- if the address is a label in the text area, then it is a warning
    // AnyJumpToData -- if any jump is to a data label, then it is a warning (label strings should have data/text prefix)
    /// An instruction is a member of more than one function.
    NodeInManyFunctions(ParserNode, Vec<Rc<Function>>),
    DoubleStoreInst {
        current_node: ParserNode,
        offset_to_use: i32,
    },
}

#[derive(Clone)]
pub enum SeverityLevel {
    Error,
    Warning,
    Information,
    Hint,
}

impl From<&LintError> for SeverityLevel {
    fn from(val: &LintError) -> Self {
        match val {
            LintError::DeadAssignment(_)
            | LintError::SaveToZero(_)
            | LintError::InvalidSegment(_)
            | LintError::InvalidJumpToFunction(..)
            | LintError::FirstInstructionIsFunction(..)
            | LintError::LostRegisterValue(_)
            | LintError::NodeInManyFunctions(..)
            | LintError::UnreachableCode(_) => SeverityLevel::Warning,
            LintError::UnknownEcall(_)
            | LintError::InvalidUseAfterCall(..)
            | LintError::InvalidUseBeforeAssignment(_)
            | LintError::UnknownStack(_)
            | LintError::InvalidStackPointer(_)
            | LintError::InvalidStackPosition(_, _)
            | LintError::InvalidStackOffsetUsage(_, _)
            | LintError::OverwriteCalleeSavedRegister(_)
            | LintError::DoubleStoreInst { .. } => SeverityLevel::Error,
        }
    }
}

// implement display for passerror
impl std::fmt::Display for LintError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LintError::DeadAssignment(_) => write!(f, "Unused value"),
            LintError::SaveToZero(_) => write!(f, "Saving to zero register"),
            LintError::InvalidUseAfterCall(_, func, _) => {
                write!(f, "Invalid use after call to function {}", func.name())
            }
            LintError::InvalidSegment(_) => write!(f, "Node is in the incorrect segment"),
            LintError::InvalidJumpToFunction(..) => write!(f, "Invalid jump to function"),
            LintError::FirstInstructionIsFunction(_, func) => {
                write!(f, "First instruction is in function {}", func.name())
            }
            LintError::UnknownEcall(_) => write!(f, "Unknown ecall"),
            LintError::UnreachableCode(_) => write!(f, "Unreachable code"),
            LintError::InvalidUseBeforeAssignment(_) => write!(f, "Invalid use before assignment"),
            LintError::UnknownStack(_) => write!(f, "Unknown stack value"),
            LintError::InvalidStackPointer(_) => write!(f, "Invalid stack pointer"),
            LintError::InvalidStackPosition(_, i) => write!(
                f,
                "Invalid stack position: original sp {} {}",
                {
                    if i.is_negative() {
                        "-"
                    } else {
                        "+"
                    }
                },
                i.abs()
            ),
            LintError::OverwriteCalleeSavedRegister(_) => {
                write!(f, "Overwriting callee-saved register")
            }
            LintError::LostRegisterValue(r) => {
                write!(f, "Lost register value: {}", r.data)
            }
            LintError::InvalidStackOffsetUsage(_, i) => {
                write!(
                    f,
                    "Invalid stack offset usage: original sp {} {}",
                    {
                        if i.is_negative() {
                            "-"
                        } else {
                            "+"
                        }
                    },
                    i.abs()
                )
            }
            LintError::NodeInManyFunctions(_node, funcs) => {
                write!(
                    f,
                    "Part of multiple functions: {}",
                    funcs.iter().map(|fun| fun.name().0).join(" | ")
                )
            }
            LintError::DoubleStoreInst { .. } => {
                write!(f, "Double store instruction. This instruction could be rewritten as a single store with `str`.")
            }
        }
    }
}

impl DiagnosticMessage for LintError {
    fn level(&self) -> SeverityLevel {
        self.into()
    }
    fn title(&self) -> String {
        self.to_string()
    }
    fn description(&self) -> String {
        self.long_description()
    }
    fn long_description(&self) -> String {
        self.to_string()
    }
    fn related(&self) -> Option<Vec<super::RelatedDiagnosticItem>> {
        match self {
            LintError::InvalidUseAfterCall(_, func, call_site) => {
                Some(vec![super::RelatedDiagnosticItem {
                    file: call_site.file(),
                    range: call_site.range(),
                    description: format!("Call to function {} occurs here", func.name()),
                }])
            }
            LintError::InvalidJumpToFunction(_, jumped_from, func) => {
                Some(vec![super::RelatedDiagnosticItem {
                    file: jumped_from.file(),
                    range: jumped_from.range(),
                    description: format!("Invalid jump to function {} occurs here", func.name()),
                }])
            }
            _ => None,
        }
    }
}

// impl LintError {
//     pub fn long_description(&self) -> String {
//         match self {
//             LintError::DeadAssignment(_) => "Unused value".to_string(),
//             LintError::SaveToZero(_) => "The result of this instruction is being stored to the zero (x0) register. This instruction has no effect.".to_string(),
//             LintError::InvalidUseAfterCall(_,x) => format!("Register were read from after a function call to {}. Reading from these registers is invalid and likely contain garbage values.\n\nIt is possible that this register was not defined across every path within the function. If you expected this register to be a return value, re-examine the function definition.",
//                 x.entry.labels.iter().map(|label| label.data.0.clone()).collect::<Vec<_>>().join(", ")
//         ),
//             LintError::ImproperFuncEntry(..) => "This function can be entered through non-conventional ways. Either by the code before or through a jump. This label is treated like a function because there is either a [jal] instruction or an explicit definition of this function.".to_string(),
//             LintError::UnknownEcall(_) => "The ecall type is not recognized. It is possible that you did not set a7 to a value.".to_string(),
//             LintError::UnreachableCode(_) => "This code is unreachable. It is possible that you have a jump to a label that does not exist.".to_string(),
//             LintError::InvalidUseBeforeAssignment(_) => "This register is being used before it is assigned to.".to_string(),
//             LintError::UnknownStack(_) => "The stack value is not definitely known.".to_string(),
//             LintError::InvalidStackPointer(_) => "The stack pointer is being overwritten.".to_string(),
//             LintError::InvalidStackPosition(_, _) => "The stack value is wrong way (positive).".to_string(),
//             LintError::OverwriteCalleeSavedRegister(_, x) => format!("Register {x} is being overwritten without the original value being restored at the end of the function. This register is callee-saved and should not be overwritten.
//             You should be saving this register to the stack at the start of the function and restoring it at the end of the function."),
//             // TODO extend Overwrite with real value analysis if known
//             // You saved the value of xx to the stack on line xx. Perhaps you meant
//             // to restore from this value instead.
//         }
//     }
// }

impl DiagnosticLocation for LintError {
    fn range(&self) -> Range {
        match self {
            LintError::InvalidUseAfterCall(r, _, _)
            | LintError::SaveToZero(r)
            | LintError::InvalidUseBeforeAssignment(r)
            | LintError::LostRegisterValue(r)
            | LintError::OverwriteCalleeSavedRegister(r)
            | LintError::DeadAssignment(r) => r.pos.clone(),
            LintError::InvalidJumpToFunction(r, _, _)
            | LintError::FirstInstructionIsFunction(r, _)
            | LintError::UnknownEcall(r)
            | LintError::UnreachableCode(r)
            | LintError::InvalidSegment(r)
            | LintError::UnknownStack(r)
            | LintError::InvalidStackPointer(r)
            | LintError::InvalidStackOffsetUsage(r, _)
            | LintError::NodeInManyFunctions(r, _)
            | LintError::InvalidStackPosition(r, _)
            | LintError::DoubleStoreInst {
                current_node: r, ..
            } => r.range(),
        }
    }

    fn file(&self) -> Uuid {
        match self {
            LintError::InvalidUseAfterCall(r, _, _)
            | LintError::SaveToZero(r)
            | LintError::InvalidUseBeforeAssignment(r)
            | LintError::LostRegisterValue(r)
            | LintError::OverwriteCalleeSavedRegister(r)
            | LintError::DeadAssignment(r) => r.file,
            LintError::FirstInstructionIsFunction(r, _)
            | LintError::InvalidJumpToFunction(r, _, _)
            | LintError::UnknownEcall(r)
            | LintError::InvalidSegment(r)
            | LintError::UnreachableCode(r)
            | LintError::UnknownStack(r)
            | LintError::InvalidStackPointer(r)
            | LintError::InvalidStackOffsetUsage(r, _)
            | LintError::NodeInManyFunctions(r, _)
            | LintError::InvalidStackPosition(r, _)
            | LintError::DoubleStoreInst {
                current_node: r, ..
            } => r.file(),
        }
    }
}
