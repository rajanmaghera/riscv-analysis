use crate::analysis::AvailableValue;
use crate::analysis::MemoryLocation;
use crate::parser::InstructionProperties;
use crate::parser::LabelStringToken;
use crate::parser::ParserNode;
use crate::parser::Register;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use super::environment_in_outs;
use super::AvailableValueMap;
use super::Cfg;
use super::Function;
use super::RefCellReplacement;
use super::RegisterSet;
use super::Segment;

#[derive(Debug)]
pub struct CfgNode {
    /// Parser node that this CFG node is wrapping.
    node: RefCell<ParserNode>,
    /// Any labels that refer to this instruction.
    pub labels: HashSet<LabelStringToken>,
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
    pub fn new(node: ParserNode, labels: HashSet<LabelStringToken>, segment: Segment) -> Self {
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

    #[must_use]
    pub fn set_node(&self, node: ParserNode) -> bool {
        self.node.replace_if_changed(node)
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

    #[must_use]
    pub fn set_reg_values_in(&self, available_in: AvailableValueMap<Register>) -> bool {
        self.reg_values_in.replace_if_changed(available_in)
    }

    pub fn reg_values_out(&self) -> AvailableValueMap<Register> {
        self.reg_values_out.borrow().clone()
    }

    #[must_use]
    pub fn set_reg_values_out(&self, available_out: AvailableValueMap<Register>) -> bool {
        self.reg_values_out.replace_if_changed(available_out)
    }

    pub fn memory_values_in(&self) -> AvailableValueMap<MemoryLocation> {
        self.memory_values_in.borrow().clone()
    }

    #[must_use]
    pub fn set_memory_values_in(&self, memory_in: AvailableValueMap<MemoryLocation>) -> bool {
        self.memory_values_in.replace_if_changed(memory_in)
    }

    pub fn memory_values_out(&self) -> AvailableValueMap<MemoryLocation> {
        self.memory_values_out.borrow().clone()
    }

    #[must_use]
    pub fn set_memory_values_out(&self, memory_out: AvailableValueMap<MemoryLocation>) -> bool {
        self.memory_values_out.replace_if_changed(memory_out)
    }

    pub fn live_in(&self) -> RegisterSet {
        *self.live_in.borrow()
    }

    #[must_use]
    pub fn set_live_in(&self, live_in: RegisterSet) -> bool {
        self.live_in.replace_if_changed(live_in)
    }

    pub fn live_out(&self) -> RegisterSet {
        *self.live_out.borrow()
    }

    #[must_use]
    pub fn set_live_out(&self, live_out: RegisterSet) -> bool {
        self.live_out.replace_if_changed(live_out)
    }

    pub fn u_def(&self) -> RegisterSet {
        *self.u_def.borrow()
    }

    #[must_use]
    pub fn set_u_def(&self, u_def: RegisterSet) -> bool {
        self.u_def.replace_if_changed(u_def)
    }

    pub fn calls_to_from_cfg(&self, cfg: &Cfg) -> Option<(Rc<Function>, LabelStringToken)> {
        if let Some(name) = self.calls_to() {
            cfg.functions().get(&name).cloned().map(|x| (x, name))
        } else if let Some(name) = self.is_some_jump_to_label() {
            // In some cases, functions may be called by jumping to them indirectly
            cfg.functions().get(&name).cloned().map(|x| (x, name))
        } else {
            None
        }
    }

    pub fn known_ecall(&self) -> Option<i32> {
        if self.is_ecall() {
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
    pub fn is_function_entry_with_func(&self) -> Option<Rc<Function>> {
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
        return !self.functions().is_empty();
    }

    pub fn labels(&self) -> HashSet<LabelStringToken> {
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
