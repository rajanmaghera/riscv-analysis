use crate::analysis::AvailableValue;
use crate::analysis::MemoryLocation;
use crate::parser::LabelString;
use crate::parser::ParserNode;
use crate::parser::Register;
use crate::parser::With;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use super::environment_in_outs;
use super::AvailableValueMap;
use super::Cfg;
use super::Function;
use super::RegisterSet;
use super::Segment;

#[derive(Debug)]
pub struct CfgNode {
    /// Parser node that this CFG node is wrapping.
    node: RefCell<ParserNode>,
    /// Any labels that refer to this instruction.
    pub labels: HashSet<With<LabelString>>,
    /// Which segment is this node in?
    segment: Segment,
    /// CFG nodes that come after this one (forward edges).
    nexts: RefCell<HashSet<Rc<CfgNode>>>,
    /// CFG nodes that come before this one (backward edges).
    prevs: RefCell<HashSet<Rc<CfgNode>>>,
    /// Which functions, if any, is this node a part of.
    ///
    /// Note that a node could be a part of 0, 1 or more functions.
    function: RefCell<HashSet<Rc<Function>>>,
    /// Map each register to the available value that is set before
    /// the instruction represented by this CFG node is run.
    ///
    /// The maps will contain all known registers and their known
    /// values. This means that many values might be duplicated above
    /// and below this CFG node.
    reg_values_in: RefCell<AvailableValueMap<Register>>,
    /// Map each register to the available value that is set after
    /// the instruction represented by this CFG node is run.
    ///
    /// The maps will contain all known registers and their known
    /// values. This means that many values might be duplicated above
    /// and below this CFG node.
    reg_values_out: RefCell<AvailableValueMap<Register>>,
    /// Map each memory location to the available value
    /// that is set before the instruction represented by this
    /// CFG node is run.
    memory_values_in: RefCell<AvailableValueMap<MemoryLocation>>,
    /// Map each memory location to the available value
    /// that is set after the instruction represented by this
    /// CFG node is run.
    memory_values_out: RefCell<AvailableValueMap<MemoryLocation>>,
    /// The set of registers that are live before the instruction
    /// represented by this CFG node is run.
    live_in: RefCell<RegisterSet>,
    /// The set of registers that are live after the instruction
    /// represented by this CFG node is run.
    live_out: RefCell<RegisterSet>,
    /// The set of registers that have unconditionally been set after
    /// the instruction represented by this CFG node is run.
    ///
    /// Registers that are unconditionally set are those that, no matter
    /// what control flow is taken, will always be set to some value. For
    /// this field, we do not care what the value is set to. We only care
    /// about whether it has been set/overwritten.
    ///
    /// Unconditionally set registers must be set in every path. For example,
    /// if there is a divergent if-else branch and the target block sets a register,
    /// that register will be contained in the `u_def` set at the end of the function
    /// if it is also set in the fallthrough block.
    ///
    /// Unconditionally set registers are used to determine the set of registers
    /// that might be return values. A return value register must be unconditionally
    /// set by the time a function returns.
    u_def: RefCell<RegisterSet>,
}

impl CfgNode {
    #[must_use]
    pub fn new(node: ParserNode, labels: HashSet<With<LabelString>>, segment: Segment) -> Self {
        CfgNode {
            node: RefCell::new(node),
            labels,
            segment,
            nexts: RefCell::new(HashSet::new()),
            prevs: RefCell::new(HashSet::new()),
            function: RefCell::new(HashSet::new()),
            reg_values_in: RefCell::new(AvailableValueMap::new()),
            reg_values_out: RefCell::new(AvailableValueMap::new()),
            memory_values_in: RefCell::new(AvailableValueMap::new()),
            memory_values_out: RefCell::new(AvailableValueMap::new()),
            live_in: RefCell::new(RegisterSet::new()),
            live_out: RefCell::new(RegisterSet::new()),
            u_def: RefCell::new(RegisterSet::new()),
        }
    }

    pub fn set_node(&self, node: ParserNode) {
        *self.node.borrow_mut() = node;
    }

    pub fn node(&self) -> ParserNode {
        self.node.borrow().clone()
    }

    pub fn nexts(&self) -> Ref<HashSet<Rc<CfgNode>>> {
        self.nexts.borrow()
    }

    pub fn prevs(&self) -> Ref<HashSet<Rc<CfgNode>>> {
        self.prevs.borrow()
    }

    /// Return the functions that this node belongs to.
    pub fn functions(&self) -> Ref<HashSet<Rc<Function>>> {
        self.function.borrow()
    }

    /// Mark this node as belonging to a given function. Each node can belong to
    /// more than one function.
    pub fn insert_function(&self, function: Rc<Function>) {
        (*self.function.borrow_mut()).insert(function);
    }

    pub fn reg_values_in(&self) -> AvailableValueMap<Register> {
        self.reg_values_in.borrow().clone()
    }

    pub fn set_reg_values_in(&self, available_in: AvailableValueMap<Register>) {
        *self.reg_values_in.borrow_mut() = available_in;
    }

    pub fn reg_values_out(&self) -> AvailableValueMap<Register> {
        self.reg_values_out.borrow().clone()
    }

    pub fn set_reg_values_out(&self, available_out: AvailableValueMap<Register>) {
        *self.reg_values_out.borrow_mut() = available_out;
    }

    pub fn memory_values_in(&self) -> AvailableValueMap<MemoryLocation> {
        self.memory_values_in.borrow().clone()
    }

    pub fn set_memory_values_in(&self, memory_in: AvailableValueMap<MemoryLocation>) {
        *self.memory_values_in.borrow_mut() = memory_in;
    }

    pub fn memory_values_out(&self) -> AvailableValueMap<MemoryLocation> {
        self.memory_values_out.borrow().clone()
    }

    pub fn set_memory_values_out(&self, memory_out: AvailableValueMap<MemoryLocation>) {
        *self.memory_values_out.borrow_mut() = memory_out;
    }

    pub fn live_in(&self) -> RegisterSet {
        *self.live_in.borrow()
    }

    pub fn set_live_in(&self, live_in: RegisterSet) {
        *self.live_in.borrow_mut() = live_in;
    }

    pub fn live_out(&self) -> RegisterSet {
        *self.live_out.borrow()
    }

    pub fn set_live_out(&self, live_out: RegisterSet) {
        *self.live_out.borrow_mut() = live_out;
    }

    pub fn u_def(&self) -> RegisterSet {
        *self.u_def.borrow()
    }

    pub fn set_u_def(&self, u_def: RegisterSet) {
        *self.u_def.borrow_mut() = u_def;
    }

    pub fn calls_to(&self, cfg: &Cfg) -> Option<(Rc<Function>, With<LabelString>)> {
        if let Some(name) = self.node().calls_to() {
            cfg.functions().get(&name).cloned().map(|x| (x, name))
        } else {
            None
        }
    }

    pub fn known_ecall(&self) -> Option<i32> {
        if self.node().is_ecall() {
            if let Some(AvailableValue::Constant(call_num)) =
                self.reg_values_in().get(&Register::ecall_type())
            {
                return Some(*call_num);
            }
        }
        None
    }

    pub fn known_ecall_signature(&self) -> Option<(RegisterSet, RegisterSet)> {
        if let Some(call_num) = self.known_ecall() {
            if let Some((ins, out)) = environment_in_outs(call_num) {
                return Some((ins, out));
            }
        }
        None
    }

    pub fn is_program_exit(&self) -> bool {
        self.known_ecall() == Some(10) || self.known_ecall() == Some(93)
    }

    pub fn insert_next(&self, next: Rc<CfgNode>) {
        self.nexts.borrow_mut().insert(next);
    }

    pub fn remove_next(&self, next: &Rc<CfgNode>) {
        self.nexts.borrow_mut().remove(next);
    }

    pub fn clear_nexts(&self) {
        self.nexts.borrow_mut().clear();
    }

    pub fn insert_prev(&self, prev: Rc<CfgNode>) {
        self.prevs.borrow_mut().insert(prev);
    }

    pub fn remove_prev(&self, prev: &Rc<CfgNode>) {
        self.prevs.borrow_mut().remove(prev);
    }

    pub fn clear_prevs(&self) {
        self.prevs.borrow_mut().clear();
    }

    /// If this node is an entry point, return the corresponding function.
    pub fn is_function_entry(&self) -> Option<Rc<Function>> {
        for func in self.functions().iter() {
            let func = Rc::clone(func);
            if &*func.entry() == self {
                return Some(func);
            }
        }
        None
    }

    /// Return true if this node is part of a function.
    pub fn is_part_of_some_function(&self) -> bool {
        return self.functions().len() > 0;
    }

    pub fn labels(&self) -> HashSet<With<LabelString>> {
        self.labels.clone()
    }

    /// Get the segment that this node is in.
    ///
    /// The segment is the section of the program that this node is in.
    /// For example, the `.text` section or the `.data` section.
    pub fn segment(&self) -> Segment {
        self.segment
    }
}

impl Hash for CfgNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node().hash(state);
    }
}

impl PartialEq for CfgNode {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}
impl Eq for CfgNode {}
