use crate::cfg::{BasicBlock, CFG};
use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range};
use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

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

pub enum WarningLevel {
    Suggestion,
    Warning,
    Error,
}

impl Into<WarningLevel> for &PassError {
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
                if let Some(register) = (*node).stores_to() {
                    if register == Register::X0 {
                        errors.push(PassError::SaveToZero(register.get_range()));
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

// TODO should I be storing this map inside the blocks?
type DirectionMap = HashMap<Rc<BasicBlock>, Direction>;
struct Direction {
    next: HashSet<Rc<BasicBlock>>,
    prev: HashSet<Rc<BasicBlock>>,
}

type LabelMap = HashMap<String, Rc<BasicBlock>>;

trait DirectionalCFG {
    fn calculate_directions(&self) -> DirectionMap;
    fn calculate_labels(&self) -> LabelMap;
}

struct UseDef {
    use_: HashSet<Register>,
    def: HashSet<Register>,
}

struct GenKill {
    gen: HashSet<Register>,
    kill: HashSet<Register>,
}
// TODO left off here

impl DirectionalCFG for CFG {
    fn calculate_labels(&self) -> LabelMap {
        self.labels.clone()
    }

    fn calculate_directions(&self) -> DirectionMap {
        // initialize the direction map
        let mut direction_map = DirectionMap::new();
        for block in self.blocks.clone() {
            direction_map.insert(
                block.clone(),
                Direction {
                    next: HashSet::new(),
                    prev: HashSet::new(),
                },
            );
        }

        // calculate next and previous of each BasicBlock with linear scan
        let mut prev: Option<Rc<BasicBlock>> = None;
        for block in self.blocks.clone() {
            if let Some(prev) = prev {
                direction_map
                    .get_mut(&prev)
                    .unwrap()
                    .next
                    .insert(block.clone());
                direction_map
                    .get_mut(&block)
                    .unwrap()
                    .prev
                    .insert(prev.clone());
            }
            prev = Some(block.clone());
        }

        // TODO find all targets of branches and add them to the next set
        // If we have made our CFG correctly, all possible branches should only
        // ever be at the end of a block, so we can just look at the last node
        // of each block

        direction_map
    }
}
// tests for DirectionalCFG
#[cfg(test)]
mod tests {
    use crate::cfg::CFG;

    use super::*;
    use std::str::FromStr;

    #[test]
    fn has_prev_and_before_items() {
        let blocks = CFG::from_str("add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2\naddi, s1, s1, 0x1").expect("unable to create cfg");

        let block1 = blocks.blocks.get(0).unwrap();
        let block2 = blocks.blocks.get(1).unwrap();
        let block3 = blocks.blocks.get(2).unwrap();

        let map = blocks.calculate_directions();
        assert_eq!(map.get(block1).unwrap().prev.len(), 0);
        assert_eq!(map.get(block1).unwrap().next.len(), 1);
        assert_eq!(map.get(block2).unwrap().prev.len(), 1);
        assert_eq!(map.get(block2).unwrap().next.len(), 1);
        assert_eq!(map.get(block3).unwrap().prev.len(), 1);
        assert_eq!(map.get(block3).unwrap().next.len(), 0);

        assert_eq!(map.get(block1).unwrap().next.get(block2).unwrap(), block2);
        assert_eq!(map.get(block2).unwrap().prev.get(block1).unwrap(), block1);
        assert_eq!(map.get(block2).unwrap().next.get(block3).unwrap(), block3);
        assert_eq!(map.get(block3).unwrap().prev.get(block2).unwrap(), block2);
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
