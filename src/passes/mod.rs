use crate::cfg::{BasicBlock, CFG};
use crate::parser::ast::{self, ASTNode, NodeData};
use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
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

pub trait DirectionalCFG {
    fn calculate_directions(&self) -> DirectionalWrapper<'_>;
    fn calculate_labels(&self) -> LabelMap;
}

pub struct DirectionalWrapper<'a> {
    cfg: &'a CFG,
    directions: DirectionMap,
}

pub trait UseDefItems {
    fn uses(&self) -> HashSet<Register>;
    fn defs(&self) -> HashSet<Register>;
}

impl UseDefItems for ASTNode {
    fn defs(&self) -> HashSet<Register> {
        let reg = match self.to_owned() {
            ASTNode::Arith(x) => Some(x.rd),
            ASTNode::IArith(x) => Some(x.rd),
            ASTNode::UpperArith(x) => Some(x.rd),
            ASTNode::Label(_) => None,
            ASTNode::JumpLink(x) => Some(x.rd),
            ASTNode::JumpLinkR(x) => Some(x.rd),
            ASTNode::Basic(_) => None,
            ASTNode::Directive(_) => None,
            ASTNode::Branch(_) => None,
            ASTNode::Store(_) => None,
            ASTNode::Load(x) => Some(x.rd),
            ASTNode::LoadAddr(x) => Some(x.rd),
            ASTNode::CSR(x) => Some(x.rd),
            ASTNode::CSRImm(x) => Some(x.rd),
        };
        // skip x0-x4
        if let Some(reg) = reg {
            if reg == Register::X0
                || reg == Register::X1
                || reg == Register::X2
                || reg == Register::X3
                || reg == Register::X4
            {
                HashSet::new()
            } else {
                let mut set = HashSet::new();
                set.insert(reg.data);
                set
            }
        } else {
            HashSet::new()
        }
    }
    fn uses(&self) -> HashSet<Register> {
        let regs: HashSet<Register> = match self {
            ASTNode::Arith(x) => vec![x.rs1.data, x.rs2.data].into_iter().collect(),
            ASTNode::IArith(x) => vec![x.rs1.data].into_iter().collect(),
            ASTNode::UpperArith(x) => HashSet::new(),
            ASTNode::Label(_) => HashSet::new(),
            ASTNode::JumpLink(x) => HashSet::new(),
            ASTNode::JumpLinkR(x) => vec![x.rs1.data].into_iter().collect(),
            ASTNode::Basic(_) => HashSet::new(),
            ASTNode::Directive(_) => HashSet::new(),
            ASTNode::Branch(x) => vec![x.rs1.data, x.rs2.data].into_iter().collect(),
            ASTNode::Store(x) => vec![x.rs1.data, x.rs2.data].into_iter().collect(),
            ASTNode::Load(x) => vec![x.rs1.data].into_iter().collect(),
            ASTNode::LoadAddr(_) => HashSet::new(),
            ASTNode::CSR(x) => vec![x.rs1.data].into_iter().collect(),
            ASTNode::CSRImm(_) => HashSet::new(),
        };
        // filter out x0 to x4
        regs.into_iter()
            .filter(|x| {
                *x != Register::X0
                    && *x != Register::X1
                    && *x != Register::X2
                    && *x != Register::X3
                    && *x != Register::X4
            })
            .collect::<HashSet<_>>()
    }
}

trait InOutRegs {
    fn in_regs(&self) -> HashSet<Register>;
    fn out_regs(&self) -> HashSet<Register>;
}

trait ToRegBitmap {
    fn to_bitmap(&self) -> u32;
}

trait ToRegHashset {
    fn to_hashset(&self) -> HashSet<Register>;
}

impl ToRegBitmap for HashSet<Register> {
    fn to_bitmap(&self) -> u32 {
        convert_to_bitmap(self.clone())
    }
}

impl ToRegHashset for u32 {
    fn to_hashset(&self) -> HashSet<Register> {
        convert_to_hashset(*self)
    }
}

fn convert_to_hashset(bitmap: u32) -> HashSet<Register> {
    let mut set = HashSet::new();
    for i in 0..32 {
        if bitmap & (1 << i) != 0 {
            set.insert(Register::from_num(i));
        }
    }
    set
}

fn convert_to_bitmap(set: HashSet<Register>) -> u32 {
    let mut bitmap = 0;
    for reg in set {
        bitmap |= 1 << reg.to_num();
    }
    bitmap
}

// calculate the in and out registers for every statement
impl DirectionalWrapper<'_> {
    pub fn calculate_in_out(&self) -> () {
        // initialize the in and out registers for every statement
        // TODO switch to structs that are a bit more typesafe
        let mut defs = Vec::new();
        let mut uses = Vec::new();
        let mut ins = Vec::new();
        let mut outs = Vec::new();
        let mut nexts = Vec::new();
        let mut prevs = Vec::new();
        let mut astidx = HashMap::new();
        for block in &self.cfg.blocks {
            let len = block.0.len();
            for (i, node) in block.0.iter().enumerate() {
                // TODO ensure basic block cannot be empty
                astidx.insert(node.clone(), i);

                // determine previous of each node
                if i == 0 {
                    let block = self.directions.get(block).unwrap().prev.clone();
                    let mut prev = HashSet::new();
                    for item in block {
                        prev.insert(item.0.last().unwrap().to_owned());
                    }
                    prevs.push(prev);
                } else {
                    let mut prev = HashSet::new();
                    prev.insert(block.0[i - 1].clone());
                    prevs.push(prev);
                }

                // determine next of each node
                if i == len - 1 {
                    let block = self.directions.get(block).unwrap().next.clone();
                    let mut next = HashSet::new();
                    for item in block {
                        next.insert(item.0.first().unwrap().to_owned());
                    }
                    nexts.push(next);
                } else {
                    let mut next = HashSet::new();
                    next.insert(block.0[i + 1].clone());
                    nexts.push(next);
                }

                uses.push(node.uses().to_bitmap());
                defs.push(node.defs().to_bitmap());
                ins.push(0);
                outs.push(0);
            }
        }

        // calculate the in and out registers for every statement
        let mut changed = true;
        while changed {
            changed = false;
            let len = defs.len();
            for i in 0..len {
                // get union of IN of all successors of this node
                let mut out = 0;
                for next in &nexts[i] {
                    let idx = astidx.get(next).unwrap();
                    out |= ins[*idx].clone();
                }
                outs[i] = out;

                // TODO debug, this is incorrect at the moment

                // calculate new IN
                let in_old = ins[i].clone();
                ins[i] = uses[i].clone() | (outs[i].clone() & !defs[i].clone());
                if in_old != ins[i] {
                    changed = true;
                }
            }
        }

        // convert the in and out registers to hashsets
        let mut ins_hashset = Vec::new();
        let mut outs_hashset = Vec::new();
        for i in 0..ins.len() {
            ins_hashset.push(ins[i].to_hashset());
            outs_hashset.push(outs[i].to_hashset());
        }

        // print the in and out registers
        for (i, block) in self.cfg.blocks.iter().enumerate() {
            println!("BLOCK: {}", i);
            for (j, node) in block.0.iter().enumerate() {
                println!(
                    "IN: {:?}, OUT: {:?}, USES: {:?}, DEFS: {:?}",
                    ins_hashset[i * block.0.len() + j],
                    outs_hashset[i * block.0.len() + j],
                    node.uses(),
                    node.defs()
                );
            }
        }
    }
}

impl Display for DirectionalWrapper<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let mut labels = self.cfg.labels_for_branch.iter();

        for block in self.cfg.blocks.iter() {
            let prevvec = self
                .directions
                .get(block)
                .unwrap()
                .prev
                .iter()
                .collect::<Vec<_>>()
                .iter()
                .map(|x| x.1.as_simple().to_string()[..8].to_string())
                .collect::<Vec<_>>()
                .join(", ");
            s.push_str(&format!(
                "ID: {}, LABELS: {:?}, PREV: [{}]\n",
                block.1.as_simple().to_string()[..8].to_string(),
                labels.next().unwrap(),
                prevvec
            ));
            s.push_str("/---------\n");
            for node in block.0.iter() {
                s.push_str(&format!(
                    "| {}  [use: ({}), def: ({})]\n",
                    node,
                    node.uses()
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    node.defs()
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            s.push_str("\\--------\n");
            // convert hashset to vector for display
            let nextvec = self
                .directions
                .get(block)
                .unwrap()
                .next
                .iter()
                .collect::<Vec<_>>()
                .iter()
                .map(|x| x.1.as_simple().to_string()[..8].to_string())
                .collect::<Vec<_>>()
                .join(", ");
            s.push_str(&format!("NEXT: [{}]\n\n", nextvec));
        }
        write!(f, "{}", s)
    }
}

impl DirectionalCFG for CFG {
    fn calculate_labels(&self) -> LabelMap {
        self.labels.clone()
    }

    fn calculate_directions(&self) -> DirectionalWrapper<'_> {
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

        let mut prev: Option<Rc<BasicBlock>> = None;
        for block in self.blocks.clone() {
            for node in block.0.clone() {
                if let Some(n) = node.jumps_to() {
                    // assert that this is the final node in the block
                    // assert_eq!(block.0.last().unwrap(), &node);
                    direction_map
                        .get_mut(&block)
                        .unwrap()
                        .next
                        .insert(self.labels.get(&n.data.0).unwrap().clone());
                    direction_map
                        .get_mut(self.labels.get(&n.data.0).unwrap())
                        .unwrap()
                        .prev
                        .insert(block.clone());
                }
            }

            // if the current block ends with a halt, we don't want to add it to the previous

            // LIN-SCAN
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

            // done weird because it's unstable
            prev = if let Some(fin) = block.0.last() {
                if fin.is_halt() {
                    None
                } else {
                    Some(block.clone())
                }
            } else {
                Some(block.clone())
            }
        }

        // JUMP TARGETS
        // TODO find all targets of branches and add them to the next set
        // If we have made our CFG correctly, all possible branches should only
        // ever be at the end of a block, so we can just look at the last node
        // of each block

        DirectionalWrapper {
            cfg: self,
            directions: direction_map,
        }
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
        assert_eq!(map.directions.get(block1).unwrap().prev.len(), 0);
        assert_eq!(map.directions.get(block1).unwrap().next.len(), 1);
        assert_eq!(map.directions.get(block2).unwrap().prev.len(), 1);
        assert_eq!(map.directions.get(block2).unwrap().next.len(), 1);
        assert_eq!(map.directions.get(block3).unwrap().prev.len(), 1);
        assert_eq!(map.directions.get(block3).unwrap().next.len(), 0);
        assert_eq!(
            map.directions
                .get(block1)
                .unwrap()
                .next
                .get(block2)
                .unwrap(),
            block2
        );
        assert_eq!(
            map.directions
                .get(block2)
                .unwrap()
                .prev
                .get(block1)
                .unwrap(),
            block1
        );
        assert_eq!(
            map.directions
                .get(block2)
                .unwrap()
                .next
                .get(block3)
                .unwrap(),
            block3
        );
        assert_eq!(
            map.directions
                .get(block3)
                .unwrap()
                .prev
                .get(block2)
                .unwrap(),
            block2
        );
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
