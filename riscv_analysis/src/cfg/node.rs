use crate::analysis::AvailableValue;
use crate::parser::LabelString;
use crate::parser::ParserNode;
use crate::parser::Register;
use crate::parser::With;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

use super::environment_in_outs;
use super::Cfg;
use super::Function;

#[derive(Debug)]
pub struct CfgNode {
    /// Parser node that this CFG node is wrapping.
    node: RefCell<ParserNode>,
    /// Any labels that refer to this instruction.
    pub labels: HashSet<With<LabelString>>,
    /// Is this node inside the data section?
    pub data_section: bool,
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
    reg_values_in: RefCell<HashMap<Register, AvailableValue>>,
    /// Map each register to the available value that is set after
    /// the instruction represented by this CFG node is run.
    ///
    /// The maps will contain all known registers and their known
    /// values. This means that many values might be duplicated above
    /// and below this CFG node.
    reg_values_out: RefCell<HashMap<Register, AvailableValue>>,
    /// Map each stack location offset to the available value
    /// that is set before the instruction represented by this
    /// CFG node is run.
    ///
    /// The stack location offset is the offset added to the
    /// stack pointer to get a specific available value. For example,
    /// the integer `-8` will map to the value at address `sp - 8`.
    ///
    /// The stack pointer is always referring to the stack pointer
    /// value at the beginning of the function body.
    ///
    /// In the current implementation, only 32-bit values are
    /// kept track of on the stack. This is because the register
    /// is 32-bit.
    stack_values_in: RefCell<HashMap<i32, AvailableValue>>,
    /// Map each stack location offset to the available value
    /// that is set after the instruction represented by this
    /// CFG node is run.
    ///
    /// The stack location offset is the offset added to the
    /// stack pointer to get a specific available value. For example,
    /// the integer `-8` will map to the value at address `sp - 8`.
    ///
    /// The stack pointer is always referring to the stack pointer
    /// value at the beginning of the function body.
    ///
    /// In the current implementation, only 32-bit values are
    /// kept track of on the stack. This is because the register
    /// is 32-bit.
    stack_values_out: RefCell<HashMap<i32, AvailableValue>>,
    /// The set of registers that are live before the instruction
    /// represented by this CFG node is run.
    live_in: RefCell<HashSet<Register>>,
    /// The set of registers that are live after the instruction
    /// represented by this CFG node is run.
    live_out: RefCell<HashSet<Register>>,
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
    u_def: RefCell<HashSet<Register>>,
}

impl CfgNode {
    #[must_use]
    pub fn new(node: ParserNode, labels: HashSet<With<LabelString>>, data_section: bool) -> Self {
        CfgNode {
            node: RefCell::new(node),
            labels,
            data_section,
            nexts: RefCell::new(HashSet::new()),
            prevs: RefCell::new(HashSet::new()),
            function: RefCell::new(HashSet::new()),
            reg_values_in: RefCell::new(HashMap::new()),
            reg_values_out: RefCell::new(HashMap::new()),
            stack_values_in: RefCell::new(HashMap::new()),
            stack_values_out: RefCell::new(HashMap::new()),
            live_in: RefCell::new(HashSet::new()),
            live_out: RefCell::new(HashSet::new()),
            u_def: RefCell::new(HashSet::new()),
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

    pub fn reg_values_in(&self) -> HashMap<Register, AvailableValue> {
        self.reg_values_in.borrow().clone()
    }

    pub fn set_reg_values_in(&self, available_in: HashMap<Register, AvailableValue>) {
        *self.reg_values_in.borrow_mut() = available_in;
    }

    pub fn reg_values_out(&self) -> HashMap<Register, AvailableValue> {
        self.reg_values_out.borrow().clone()
    }

    pub fn set_reg_values_out(&self, available_out: HashMap<Register, AvailableValue>) {
        *self.reg_values_out.borrow_mut() = available_out;
    }

    pub fn stack_values_in(&self) -> HashMap<i32, AvailableValue> {
        self.stack_values_in.borrow().clone()
    }

    pub fn set_stack_values_in(&self, stack_in: HashMap<i32, AvailableValue>) {
        *self.stack_values_in.borrow_mut() = stack_in;
    }

    pub fn stack_values_out(&self) -> HashMap<i32, AvailableValue> {
        self.stack_values_out.borrow().clone()
    }

    pub fn set_stack_values_out(&self, stack_out: HashMap<i32, AvailableValue>) {
        *self.stack_values_out.borrow_mut() = stack_out;
    }

    pub fn live_in(&self) -> HashSet<Register> {
        self.live_in.borrow().clone()
    }

    pub fn set_live_in(&self, live_in: HashSet<Register>) {
        *self.live_in.borrow_mut() = live_in;
    }

    pub fn live_out(&self) -> HashSet<Register> {
        self.live_out.borrow().clone()
    }

    pub fn set_live_out(&self, live_out: HashSet<Register>) {
        *self.live_out.borrow_mut() = live_out;
    }

    pub fn u_def(&self) -> HashSet<Register> {
        self.u_def.borrow().clone()
    }

    pub fn set_u_def(&self, u_def: HashSet<Register>) {
        *self.u_def.borrow_mut() = u_def;
    }

    pub fn calls_to(&self, cfg: &Cfg) -> Option<(Rc<Function>, With<LabelString>)> {
        if let Some(name) = self.node().calls_to() {
            cfg.label_function_map
                .get(&name)
                .cloned()
                .map(|x| (x, name))
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

    pub fn known_ecall_signature(&self) -> Option<(HashSet<Register>, HashSet<Register>)> {
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
            let func = func.clone();
            if &*func.entry == self {
                return Some(func)
            }
        }

        return None;
    }

    /// Return true if this node is part of a function.
    pub fn has_function(&self) -> bool {
        return self.functions().len() > 0;
    }

    pub fn labels(&self) -> HashSet<With<LabelString>> {
        self.labels.clone()
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
