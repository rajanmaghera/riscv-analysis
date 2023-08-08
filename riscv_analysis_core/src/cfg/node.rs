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
pub struct CFGNode {
    node: RefCell<ParserNode>,
    pub labels: HashSet<With<LabelString>>,
    pub data_section: bool,
    nexts: RefCell<HashSet<Rc<CFGNode>>>,
    prevs: RefCell<HashSet<Rc<CFGNode>>>,
    function: RefCell<Option<Rc<Function>>>,
    reg_values_in: RefCell<HashMap<Register, AvailableValue>>,
    reg_values_out: RefCell<HashMap<Register, AvailableValue>>,
    stack_values_in: RefCell<HashMap<i32, AvailableValue>>,
    stack_values_out: RefCell<HashMap<i32, AvailableValue>>,
    live_in: RefCell<HashSet<Register>>,
    live_out: RefCell<HashSet<Register>>,
    u_def: RefCell<HashSet<Register>>,
}

impl CFGNode {
    #[must_use]
    pub fn new(node: ParserNode, labels: HashSet<With<LabelString>>, data_section: bool) -> Self {
        CFGNode {
            node: RefCell::new(node),
            labels,
            data_section,
            nexts: RefCell::new(HashSet::new()),
            prevs: RefCell::new(HashSet::new()),
            function: RefCell::new(None),
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

    pub fn nexts(&self) -> Ref<HashSet<Rc<CFGNode>>> {
        self.nexts.borrow()
    }

    pub fn prevs(&self) -> Ref<HashSet<Rc<CFGNode>>> {
        self.prevs.borrow()
    }

    pub fn function(&self) -> Ref<Option<Rc<Function>>> {
        self.function.borrow()
    }

    pub fn set_function(&self, function: Rc<Function>) {
        *self.function.borrow_mut() = Some(function);
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

    pub fn insert_next(&self, next: Rc<CFGNode>) {
        self.nexts.borrow_mut().insert(next);
    }

    pub fn remove_next(&self, next: &Rc<CFGNode>) {
        self.nexts.borrow_mut().remove(next);
    }

    pub fn clear_nexts(&self) {
        self.nexts.borrow_mut().clear();
    }

    pub fn insert_prev(&self, prev: Rc<CFGNode>) {
        self.prevs.borrow_mut().insert(prev);
    }

    pub fn remove_prev(&self, prev: &Rc<CFGNode>) {
        self.prevs.borrow_mut().remove(prev);
    }

    pub fn clear_prevs(&self) {
        self.prevs.borrow_mut().clear();
    }

    pub fn is_function_entry(&self) -> Option<Rc<Function>> {
        if let Some(func) = self.function().clone() {
            if &*func.entry == self {
                return Some(func);
            }
        }
        None
    }

    pub fn labels(&self) -> HashSet<With<LabelString>> {
        self.labels.clone()
    }
}

impl Hash for CFGNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node().hash(state);
    }
}

impl PartialEq for CFGNode {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}
impl Eq for CFGNode {}
