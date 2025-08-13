use super::Cfg;
use super::Function;
use super::RefCellReplacement;
use super::RegisterSet;
use super::Segment;
use super::{environment_in_outs, ExternalFunction};
use crate::analysis::{LocValueMap, Location, Value};
use crate::parser::InstructionProperties;
use crate::parser::LabelStringToken;
use crate::parser::RVInstructionNode;
use crate::parser::RVRegister;
use std::cell::Ref;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Debug)]
pub struct CfgNode {
    /// Parser node that this CFG node is wrapping.
    node: RefCell<RVInstructionNode>,
    /// Is this node used as a function entry point?
    is_function_entry: bool,
    /// Is program entry
    is_program_entry: bool,
    /// Which functions, if any, is this node a part of.
    ///
    /// Note that a node could be a part of 0, 1 or more functions.
    function: RefCell<HashSet<Rc<Function>>>,
    real_val_in: RefCell<LocValueMap>,
    real_val_out: RefCell<LocValueMap>,
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
    pub fn new(node: RVInstructionNode, is_function_entry: bool, is_program_entry: bool) -> Self {
        CfgNode {
            node: RefCell::new(node),
            is_function_entry,
            is_program_entry,
            function: RefCell::new(HashSet::new()),
            real_val_in: RefCell::new(LocValueMap::new()),
            real_val_out: RefCell::new(LocValueMap::new()),
            live_in: RefCell::new(RegisterSet::new()),
            live_out: RefCell::new(RegisterSet::new()),
            u_def: RefCell::new(RegisterSet::new()),
        }
    }

    #[must_use]
    pub fn is_program_entry(&self) -> bool {
        self.is_program_entry
    }

    #[must_use]
    pub fn is_function_entry(&self) -> bool {
        self.is_function_entry
    }

    #[must_use]
    pub fn is_any_entry(&self) -> bool {
        self.is_function_entry() || self.is_program_entry()
    }

    #[must_use]
    pub fn set_node(&self, node: RVInstructionNode) -> bool {
        self.node.replace_if_changed(node)
    }

    pub fn node(&self) -> RVInstructionNode {
        self.node.borrow().clone()
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

    #[must_use]
    pub fn set_real_val_in(&self, val_in: LocValueMap) -> bool {
        self.real_val_in.replace_if_changed(val_in)
    }

    #[must_use]
    pub fn set_real_val_out(&self, val_out: LocValueMap) -> bool {
        self.real_val_out.replace_if_changed(val_out)
    }

    pub fn real_val_in(&self) -> LocValueMap {
        self.real_val_in.borrow().clone()
    }

    pub fn real_val_out(&self) -> LocValueMap {
        self.real_val_out.borrow().clone()
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
            cfg.get_function(&name).cloned().map(|x| (x, name))
        } else if let Some(name) = self.is_some_jump_to_label() {
            // In some cases, functions may be called by jumping to them indirectly
            cfg.get_function(&name).cloned().map(|x| (x, name))
        } else {
            None
        }
    }

    pub fn calls_to_some_external_function_from_cfg(
        &self,
        cfg: &Cfg,
    ) -> Option<(ExternalFunction, LabelStringToken)> {
        if let Some(name) = self.calls_to() {
            cfg.get_external_function(&name).map(|x| (x.clone(), name))
        } else {
            None
        }
    }

    #[deprecated]
    pub fn known_ecall(&self) -> Option<i32> {
        if self.is_ecall() {
            if let Value::Const(call_num) = self
                .real_val_in()
                .get(&Location::Register(RVRegister::ecall_type()))
            {
                return Some(call_num);
            }
        }
        None
    }

    #[deprecated]
    pub fn known_ecall_signature(&self) -> Option<(RegisterSet, RegisterSet)> {
        if let Some(call_num) = self.known_ecall() {
            if let Some((ins, out)) = environment_in_outs(call_num) {
                return Some((ins, out));
            }
        }
        None
    }

    #[deprecated]
    pub fn is_program_exit(&self) -> bool {
        self.known_ecall() == Some(10) || self.known_ecall() == Some(93)
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
    #[deprecated]
    pub fn is_part_of_some_function(&self) -> bool {
        !self.functions().is_empty()
    }

    pub fn labels(&self) -> HashSet<LabelStringToken> {
        self.node().label_names().cloned().collect()
    }

    /// Get the segment that this node is in.
    ///
    /// The segment is the section of the program that this node is in.
    /// For example, the `.text` section or the `.data` section.
    pub fn segment(&self) -> Segment {
        self.node().segment()
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
