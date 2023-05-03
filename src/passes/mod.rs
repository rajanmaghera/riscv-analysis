use crate::cfg::{BasicBlock, CFG};
use crate::parser::ast::{ASTNode, RType};
use crate::parser::inst::InstType;
use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range, WithToken};
use std::collections::HashSet;
use std::ops::Deref;

// TODO switch to types that take up zero space

#[derive(Debug)]
pub struct PassErrors {
    pub errors: Vec<PassError>,
}

trait Pass {
    fn run(&self, cfg: &CFG) -> Result<(), PassErrors>;
}

#[derive(Debug)]
pub enum PassError {
    UnusedValue(Range),
    SaveToZero(Range),
}

enum WarningLevel {
    Suggestion,
    Warning,
    Error,
}

impl Into<WarningLevel> for PassError {
    fn into(self) -> WarningLevel {
        match self {
            PassError::UnusedValue(_) => WarningLevel::Suggestion,
            PassError::SaveToZero(_) => WarningLevel::Warning,
        }
    }
}

// implement display for passerror
impl std::fmt::Display for PassError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PassError::UnusedValue(_) => write!(f, "Unused value"),
            PassError::SaveToZero(_) => write!(f, "Saving to zero register"),
        }
    }
}

impl PassError {
    pub fn long_description(&self) -> String {
        match self {
            PassError::UnusedValue(_) => "Unused value".to_string(),
            PassError::SaveToZero(_) => "The result of this instruction is being stored to the zero (x0) register. This instruction has no effect.".to_string()
        }
    }

    pub fn range(&self) -> Range {
        match self {
            PassError::UnusedValue(range) => range.clone(),
            PassError::SaveToZero(range) => range.clone(),
        }
    }
}

struct SaveToZeroCheck;
impl Pass for SaveToZeroCheck {
    fn run(&self, cfg: &CFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                if let Some(register) = (*node).data.stores_to() {
                    if register == Register::X0 {
                        errors.push(PassError::SaveToZero(node.get_range()));
                    }
                }
            }
        }
        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}

pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
}

impl PassManager {
    pub fn new() -> PassManager {
        PassManager {
            passes: vec![Box::new(SaveToZeroCheck)],
        }
    }

    pub fn run(&self, cfg: CFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();
        for pass in self.passes.iter() {
            match pass.run(&cfg) {
                Ok(_) => (),
                Err(mut pass_errors) => {
                    errors.append(&mut pass_errors.errors);
                }
            }
        }
        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}
