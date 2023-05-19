use crate::cfg::{self, BasicBlock, CFG};
use crate::parser::ast::ASTNode;
use crate::parser::register::Register;
use crate::parser::token::{LineDisplay, Range};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
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
    InvalidUseAfterCall(Range, WithToken<LabelString>),
    JumpToFunc(Range, LabelString),
    NaturalFuncEntry(Range),
    DeadAssignment(Range),
    SaveToZero(Range),
    // SetBadRegister(Range, Register), -- used when setting registers that should not be set
    // OverwriteRaRegister(Range), -- used when overwriting the return address register
    // OverwriteRegister(Range, Register), -- used when overwriting a register that has not been saved
    // FallOffEnd(Range), program may fall off the end of code
}

pub enum WarningLevel {
    Suggestion,
    Warning,
    Error,
}

impl Into<WarningLevel> for &PassError {
    fn into(self) -> WarningLevel {
        match self {
            PassError::DeadAssignment(_) => WarningLevel::Suggestion,
            PassError::SaveToZero(_) => WarningLevel::Warning,
            PassError::InvalidUseAfterCall(_, _) => WarningLevel::Error,
            PassError::JumpToFunc(..) => WarningLevel::Warning,
            PassError::NaturalFuncEntry(_) => WarningLevel::Warning,
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
            PassError::JumpToFunc(..) => write!(f, "Jump to function"),
            PassError::NaturalFuncEntry(_) => write!(f, "Natural function entry"),
        }
    }
}

impl PassError {
    pub fn long_description(&self) -> String {
        match self {
            PassError::DeadAssignment(_) => "Unused value".to_string(),
            PassError::SaveToZero(_) => "The result of this instruction is being stored to the zero (x0) register. This instruction has no effect.".to_string(),
            PassError::InvalidUseAfterCall(_,x) => format!("Register were read from after a function call to {}. Reading from these registers is invalid and likely contain garbage values.",
                x.data.0
        ).to_string(),
            PassError::JumpToFunc(_, x) => format!("Label {} is both called and jumped to. This label will be treated like a function.", x.0).to_string(),
            PassError::NaturalFuncEntry(_) => "This function can be entered through non-conventional ways. Either by the code before or through a jump.".to_string(),
        }
    }

    pub fn range(&self) -> Range {
        match self {
            PassError::DeadAssignment(range) => range.clone(),
            PassError::SaveToZero(range) => range.clone(),
            PassError::InvalidUseAfterCall(range, _) => range.clone(),
            PassError::JumpToFunc(range, _) => range.clone(),
            PassError::NaturalFuncEntry(range) => range.clone(),
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

struct DeadValueCheck;
impl Pass for DeadValueCheck {
    fn run(&self, cfg: &CFG) -> Result<(), PassErrors> {
        let dcfg = cfg.calculate_directions();
        let node_next = dcfg.node_nexts();
        let in_outs = dcfg.calculate_in_out();
        let mut errors = Vec::new();
        let mut i: usize = 0;
        // recalc mapping of nodes to idx
        let mut node_idx = HashMap::new();
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                node_idx.insert(node, i);
                i += 1;
            }
        }
        let mut i = 0;
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                // check for any assignments that don't make it
                // to the end of the node
                for def in node.defs() {
                    if !in_outs.1.get(i).unwrap().contains(&def) {
                        errors.push(PassError::DeadAssignment(node.get_range().clone()));
                    }
                }

                // if the node is a call (func call), check that there
                // are not extra values in the IN of the next node
                if node.is_call() {
                    for next in node_next.get(&node).unwrap() {
                        // subtract the Current nodes' OUT from next's IN
                        let idx: usize = node_idx.get(next).unwrap().clone();
                        let mut next_in: HashSet<Register> = in_outs.0[idx].clone();
                        for out in in_outs.1.get(i).unwrap() {
                            next_in.remove(out);
                        }

                        // if there are any values left in next_in, then
                        // there are invalid uses of values after a call

                        // TODO have more specific annotations for this
                        // error
                        if next_in.len() > 0 {
                            errors.push(PassError::InvalidUseAfterCall(
                                node.get_range().clone(),
                                next_in,
                            ));
                        }
                    }
                }
                i += 1;
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
    return_label_map: HashMap<Rc<ASTNode>, WithToken<LabelString>>,
    label_return_map: HashMap<LabelString, HashSet<Rc<ASTNode>>>,
    label_call_map: HashMap<LabelString, HashSet<Rc<ASTNode>>>,
    next_ast_map: HashMap<Rc<ASTNode>, HashSet<Rc<ASTNode>>>,
    prev_ast_map: HashMap<Rc<ASTNode>, HashSet<Rc<ASTNode>>>,
}

pub trait UseDefItems {
    fn orig_defs(&self) -> HashSet<Register>;
    fn uses(&self) -> HashSet<Register>;
    fn uses_reg(&self) -> HashSet<WithToken<Register>>;
    fn defs(&self) -> HashSet<Register>;
}

impl UseDefItems for ASTNode {
    // These defs are used to help start some functional analysis
    fn orig_defs(&self) -> HashSet<Register> {
        match self.to_owned() {
            ASTNode::FuncEntry(_) => vec![
                Register::X1,
                Register::X10,
                Register::X11,
                Register::X12,
                Register::X13,
                Register::X14,
                Register::X15,
                Register::X16,
                Register::X17,
                // We also include all temporary registers
                // if they are in the OUT, they were used
                // in the function incorrectly
                Register::X5,
                Register::X6,
                Register::X7,
                Register::X28,
                Register::X29,
                Register::X30,
                Register::X31,
            ]
            .into_iter()
            .collect(),

            ASTNode::JumpLink(x) => {
                // a function call will "define" all argument registers
                // as if every a-register was used as a return value
                if x.rd.data != Register::X1 {
                    vec![x.rd.data].into_iter().collect()
                } else {
                    vec![
                        Register::X1,
                        Register::X10,
                        Register::X11,
                        // TODO technically a0 and a1 are the
                        // only return values?
                        Register::X12,
                        Register::X13,
                        Register::X14,
                        Register::X15,
                        Register::X16,
                        Register::X17,
                        Register::X5,
                        Register::X6,
                        Register::X7,
                        Register::X28,
                        Register::X29,
                        Register::X30,
                        Register::X31,
                    ]
                    .into_iter()
                    .collect()
                }
            }
            _ => self.defs(),
        }
    }

    fn defs(&self) -> HashSet<Register> {
        let regs: HashSet<Register> = match self.to_owned() {
            ASTNode::FuncEntry(_) => HashSet::new(),
            ASTNode::Arith(x) => vec![x.rd.data].into_iter().collect(),
            ASTNode::IArith(x) => vec![x.rd.data].into_iter().collect(),
            ASTNode::UpperArith(x) => vec![x.rd.data].into_iter().collect(),
            ASTNode::Label(_) => HashSet::new(),
            ASTNode::JumpLink(x) => {
                // a function call will "define" all argument registers
                // as if every a-register was used as a return value
                if x.rd.data != Register::X1 {
                    vec![x.rd.data].into_iter().collect()
                } else {
                    HashSet::new()
                }
            }
            ASTNode::JumpLinkR(x) => vec![x.rd.data].into_iter().collect(),
            ASTNode::Basic(_) => HashSet::new(),
            ASTNode::Directive(_) => HashSet::new(),
            ASTNode::Branch(_) => HashSet::new(),
            ASTNode::Store(_) => HashSet::new(),
            ASTNode::Load(x) => vec![x.rd.data].into_iter().collect(),
            ASTNode::LoadAddr(x) => vec![x.rd.data].into_iter().collect(),
            ASTNode::CSR(x) => vec![x.rd.data].into_iter().collect(),
            ASTNode::CSRImm(x) => vec![x.rd.data].into_iter().collect(),
        };
        // skip x0-x4
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
    fn uses_reg(&self) -> HashSet<WithToken<Register>> {
        let regs: HashSet<WithToken<Register>> = match self {
            ASTNode::FuncEntry(_) => HashSet::new(),
            ASTNode::Arith(x) => vec![x.rs1.clone(), x.rs2.clone()].into_iter().collect(),
            ASTNode::IArith(x) => vec![x.rs1.clone()].into_iter().collect(),
            ASTNode::UpperArith(x) => HashSet::new(),
            ASTNode::Label(_) => HashSet::new(),
            ASTNode::JumpLink(x) => {
                // A function call will "use" no argument registers
                HashSet::new()
            }
            ASTNode::JumpLinkR(x) => vec![x.rs1.clone()].into_iter().collect(),
            ASTNode::Basic(_) => HashSet::new(),
            ASTNode::Directive(_) => HashSet::new(),
            ASTNode::Branch(x) => vec![x.rs1.clone(), x.rs2.clone()].into_iter().collect(),
            ASTNode::Store(x) => vec![x.rs1.clone(), x.rs2.clone()].into_iter().collect(),
            ASTNode::Load(x) => vec![x.rs1.clone()].into_iter().collect(),
            ASTNode::LoadAddr(_) => HashSet::new(),
            ASTNode::CSR(x) => vec![x.rs1.clone()].into_iter().collect(),
            ASTNode::CSRImm(_) => HashSet::new(),
        };
        // filter out x0 to x4
        let item = regs
            .into_iter()
            .filter(|x| {
                *x != Register::X0
                    && *x != Register::X1
                    && *x != Register::X2
                    && *x != Register::X3
                    && *x != Register::X4
            })
            .collect::<HashSet<_>>();
        item
    }
    fn uses(&self) -> HashSet<Register> {
        let regs: HashSet<Register> = match self {
            ASTNode::FuncEntry(_) => HashSet::new(),
            ASTNode::Arith(x) => vec![x.rs1.data, x.rs2.data].into_iter().collect(),
            ASTNode::IArith(x) => vec![x.rs1.data].into_iter().collect(),
            ASTNode::UpperArith(x) => HashSet::new(),
            ASTNode::Label(_) => HashSet::new(),
            ASTNode::JumpLink(x) => {
                // A function call will "use" no argument registers
                HashSet::new()
            }
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
        let item = regs
            .into_iter()
            .filter(|x| {
                *x != Register::X0
                    && *x != Register::X1
                    && *x != Register::X2
                    && *x != Register::X3
                    && *x != Register::X4
            })
            .collect::<HashSet<_>>();
        item
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

impl CFG {
    pub fn calc_ast_directions(
        &self,
        direction_map: &HashMap<Rc<BasicBlock>, Direction>,
    ) -> (
        HashMap<Rc<ASTNode>, HashSet<Rc<ASTNode>>>,
        HashMap<Rc<ASTNode>, HashSet<Rc<ASTNode>>>,
    ) {
        let mut nexts = HashMap::new();
        let mut prevs = HashMap::new();
        for block in &self.blocks {
            let len = block.0.len();
            for (i, node) in block.0.iter().enumerate() {
                // determine next of each node
                let mut set = HashSet::new();
                if i == len - 1 {
                    let block = direction_map.get(block).unwrap().next.clone();
                    for next in block {
                        set.insert(next.0.first().unwrap().clone());
                    }
                } else {
                    set.insert(block.0[i + 1].clone());
                }
                nexts.insert(node.clone(), set);

                // determine prevs of each node
                let mut set = HashSet::new();
                if i == 0 {
                    let block = direction_map.get(block).unwrap().prev.clone();
                    for prev in block {
                        set.insert(prev.0.last().unwrap().clone());
                    }
                } else {
                    set.insert(block.0[i - 1].clone());
                }
                prevs.insert(node.clone(), set);
            }
        }
        (nexts, prevs)
    }
}

impl DirectionalWrapper<'_> {
    pub fn node_nexts(&self) -> HashMap<Rc<ASTNode>, HashSet<Rc<ASTNode>>> {
        let mut nexts = HashMap::new();
        for block in &self.cfg.blocks {
            let len = block.0.len();
            for (i, node) in block.0.iter().enumerate() {
                // determine next of each node
                let mut set = HashSet::new();
                if i == len - 1 {
                    let block = self.directions.get(block).unwrap().next.clone();
                    for next in block {
                        set.insert(next.0.first().unwrap().clone());
                    }
                } else {
                    set.insert(block.0[i + 1].clone());
                }
                nexts.insert(node.clone(), set);
            }
        }
        nexts
    }
    pub fn calculate_in_out(&self) -> (Vec<HashSet<Register>>, Vec<HashSet<Register>>) {
        // initialize the in and out registers for every statement
        // TODO switch to structs that are a bit more typesafe
        let mut defs = Vec::new();
        let mut uses = Vec::new();
        let mut ins = Vec::new();
        let mut outs = Vec::new();

        let mut nexts = Vec::new();
        let mut ast = Vec::new();
        let mut astidx = HashMap::new();

        let mut big_idx = 0;
        for block in &self.cfg.blocks {
            for node in block.0.iter() {
                // TODO ensure basic block cannot be empty
                ast.push(node.clone());
                astidx.insert(node.clone(), big_idx);
                nexts.push(self.next_ast_map.get(node).unwrap().clone());
                uses.push(node.uses().to_bitmap());
                defs.push(node.orig_defs().to_bitmap());
                ins.push(0);
                outs.push(0);

                big_idx += 1;
            }
        }

        // HELPER VALUES
        // mask of only argument registers
        let arg_mask = vec![
            Register::X10,
            Register::X11,
            Register::X12,
            Register::X13,
            Register::X14,
            Register::X15,
            Register::X16,
            Register::X17,
        ]
        .into_iter()
        .collect::<HashSet<_>>()
        .to_bitmap();

        // calculate the in and out registers for every statement
        let mut rounds = 0;
        while rounds < 3 {
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

                    // calculate new IN
                    let in_old = ins[i].clone();
                    ins[i] = uses[i].clone() | (outs[i].clone() & !defs[i].clone());
                    if in_old != ins[i] {
                        changed = true;
                    }
                }
            }

            match rounds {
                0 => {
                    // AFTER FIRST ROUND -- INFER FUNCTION RETURN VALUES
                    for node in &ast {
                        match node.as_ref() {
                            ASTNode::JumpLink(x) => {
                                // FUNCTION CALL
                                // if an argument value is in the OUT of a use, it is a return value
                                // attach it to the use of the return statements

                                // get OUT of func
                                let idx = astidx.get(node).unwrap();
                                let out = outs[*idx].clone();
                                // AND with arg mask
                                let out = out & arg_mask;

                                // debug -- print out the registers that are used as arguments
                                println!(
                                    "Function call: {}, Guessed return values: {}",
                                    x.name.data.0,
                                    out.to_hashset()
                                        .into_iter()
                                        .fold(String::new(), |acc, x| acc
                                            + &format!("{}, ", x.to_string()))
                                );

                                // find all return statements
                                let returns = self.label_return_map.get(&x.name.data).unwrap();
                                for ret in returns {
                                    // set the use of the return statement to the return value
                                    let idx = astidx.get(ret).unwrap();
                                    uses[*idx] = out.clone();
                                }

                                // TODO if orig_out has temp registers, then they were used incorrectly

                                // TODO if we guess that a function returns a register that is never set in
                                // the function, then that register should be removed from the set

                                // TODO reset everything completely at the end of this round!!!
                                // reset defs of function call to empty
                                defs[*astidx.get(node).unwrap()] = 0;
                            }
                            _ => {}
                        }
                    }
                }
                1 => {
                    for node in &ast {
                        match node.as_ref() {
                            ASTNode::FuncEntry(x) => {
                                // FUNCTION ENTRY
                                // if an argument value is in the out of the start of a function,
                                // then attach it to the use of all function calls

                                // get OUT of start
                                let idx = astidx.get(node).unwrap();
                                let out = outs[*idx].clone();
                                // AND with arg mask
                                let out = out & arg_mask;

                                // debug -- print out the registers that are used as arguments
                                println!(
                                    "Function entry: {}, Guessed argument values: {}",
                                    x.name.data.0,
                                    out.to_hashset()
                                        .into_iter()
                                        .fold(String::new(), |acc, x| acc
                                            + &format!("{}, ", x.to_string()))
                                );

                                // find all function calls
                                let calls = self.label_call_map.get(&x.name.data).unwrap();
                                for call in calls {
                                    // set the use of the call to the guessed argument values
                                    let idx = astidx.get(call).unwrap();
                                    uses[*idx] = out.clone();
                                }

                                defs[*astidx.get(node).unwrap()] = 0;

                                // TODO if we guess that a function takes in a value that is not
                                // set before a function call, then we should return an error OR
                                // remove it from the set
                                // This error handling needs to be more robust once we get to multiple
                                // sites of entry and exit
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    break;
                }
            }
            rounds += 1;
        }

        // ARGUMENT GUESSING
        //

        // convert the in and out registers to hashsets
        let mut ins_hashset = Vec::new();
        let mut outs_hashset = Vec::new();
        for i in 0..ins.len() {
            ins_hashset.push(ins[i].to_hashset());
            outs_hashset.push(outs[i].to_hashset());
        }

        // print the in and out registers
        let mut i = 0;
        for (ii, block) in self.cfg.blocks.iter().enumerate() {
            println!("BLOCK: {}", ii);
            for (_, node) in block.0.iter().enumerate() {
                println!(
                    "IN: {:?}, OUT: {:?}, USES: {:?}, DEFS: {:?}",
                    ins_hashset[i],
                    outs_hashset[i],
                    node.uses(),
                    node.defs()
                );
                i += 1;
            }
        }
        (ins_hashset, outs_hashset)
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
                if fin.is_return() {
                    None
                } else {
                    Some(block.clone())
                }
            } else {
                Some(block.clone())
            }
        }

        // --- POST-DIRECTIONAL CALCULATIONS ---

        // AST NEXTS/PREVS
        // Using the big block nexts and prevs, we can now calculate the
        // nexts and prevs for each AST node. We do this by walking through
        let (next_ast_map, prev_ast_map) = self.calc_ast_directions(&direction_map);

        // TODO verify!!!
        // RETURN LABEL TARGETS
        // All return labels should only have one possible function entry
        // for good code, so we can just walk backwards from all return
        // labels till we reach an AST function start node. If we reach
        // multiple, we have a problem.
        let mut return_label_map = HashMap::new();
        let mut label_return_map = HashMap::new();
        // for each return label
        for block in self.blocks.clone() {
            for node in &block.0.clone() {
                if node.is_return() {
                    // walk backwards
                    let mut walked = HashSet::new();
                    let mut queue = vec![node.clone()];
                    let mut found = Vec::new();
                    'inn: while let Some(n) = queue.pop() {
                        walked.insert(n.clone());
                        // if we find a function start, we're done
                        match n.as_ref() {
                            ASTNode::FuncEntry(x) => {
                                return_label_map.insert(node.clone(), x.name.clone());
                                match label_return_map.get_mut(&x.name.data) {
                                    None => {
                                        let mut new_set = HashSet::new();
                                        new_set.insert(node.clone());
                                        label_return_map.insert(x.name.data.clone(), new_set);
                                    }
                                    Some(x) => {
                                        x.insert(node.clone());
                                    }
                                }
                                found.push(n);
                                continue 'inn;
                            }
                            _ => (),
                        }
                        // otherwise, add all prevs to the queue
                        for prev in prev_ast_map.get(&n).unwrap() {
                            if !walked.contains(prev) {
                                queue.push(prev.clone());
                            }
                        }
                    }
                    // if we found more than one, we have a problem
                    if found.len() > 1 {
                        unimplemented!("Multiple function starts found for return label");
                    } else if found.len() == 0 {
                        unimplemented!("No function starts found for return label");
                    }
                }
            }
        }

        // LABEL CALL MAP
        // Find all places where a label is called and add them to the
        // label call map
        let mut label_call_map = HashMap::new();
        for block in self.blocks.clone() {
            for node in &block.0.clone() {
                if let ASTNode::JumpLink(x) = node.as_ref() {
                    match label_call_map.get_mut(&x.name.data) {
                        None => {
                            let mut new_set = HashSet::new();
                            new_set.insert(node.clone());
                            label_call_map.insert(x.name.data.clone(), new_set);
                        }
                        Some(x) => {
                            x.insert(node.clone());
                        }
                    }
                }
            }
        }

        // JUMP TARGETS
        // TODO find all targets of branches and add them to the next set
        // If we have made our CFG correctly, all possible branches should only
        // ever be at the end of a block, so we can just look at the last node
        // of each block

        // calculate the possible function labels

        DirectionalWrapper {
            cfg: self,
            next_ast_map,
            prev_ast_map,
            return_label_map,
            label_return_map,
            label_call_map,
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
    fn next_node_from_big_nexts() {
        let str =
            "sample_eval:\nli t0, 7\nbne a0, t0, L2\nli a0, 99\nret\nL2:\naddi a0, a0, 21\nret";
        let blocks = CFG::from_str(str).expect("unable to create cfg");
        let map = blocks.calculate_directions();
        let next = map.node_nexts();

        assert_eq!(next.len(), 6);
        assert_eq!(
            next[&blocks.blocks[0].0[0]],
            HashSet::from([blocks.blocks[0].0[1].clone()])
        );
        assert_eq!(
            next[&blocks.blocks[0].0[1]],
            HashSet::from([blocks.blocks[1].0[0].clone(), blocks.blocks[2].0[0].clone(),])
        );
        assert_eq!(
            next[&blocks.blocks[1].0[0]],
            HashSet::from([blocks.blocks[1].0[1].clone()])
        );
        assert_eq!(next[&blocks.blocks[1].0[1]], HashSet::from([]));

        assert_eq!(
            next[&blocks.blocks[2].0[0]],
            HashSet::from([blocks.blocks[2].0[1].clone()])
        );
        assert_eq!(next[&blocks.blocks[2].0[1]], HashSet::from([]));
    }

    #[test]
    fn basic_live_in_out() {
        use Register::*;
        let str =
            "sample_eval:\nli t0, 7\nbne a0, t0, L2\nli a0, 99\nret\nL2:\naddi a0, a0, 21\nret";
        let blocks = CFG::from_str(str).expect("unable to create cfg");
        let map = blocks.calculate_directions();
        let (ins, outs) = map.calculate_in_out();

        assert_eq!(ins.len(), 6);
        assert_eq!(outs.len(), 6);

        assert_eq!(ins[0], HashSet::from([X10]));
        assert_eq!(outs[0], HashSet::from([X5, X10]));

        assert_eq!(ins[1], HashSet::from([X10, X5]));
        assert_eq!(outs[1], HashSet::from([X10]));

        assert_eq!(ins[2], HashSet::from([]));
        assert_eq!(outs[2], HashSet::from([X10]));

        assert_eq!(ins[3], HashSet::from([X10]));
        assert_eq!(outs[3], HashSet::from([]));

        assert_eq!(ins[4], HashSet::from([X10]));
        assert_eq!(outs[4], HashSet::from([X10]));

        assert_eq!(ins[5], HashSet::from([X10]));
        assert_eq!(outs[5], HashSet::from([]));
    }

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
            passes: vec![Box::new(SaveToZeroCheck), Box::new(DeadValueCheck)],
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
