use super::RefCellReplacement;
use std::{
    cell::{Ref, RefCell},
    collections::HashSet,
    hash::Hash,
    rc::Rc,
};
use uuid::Uuid;

use crate::parser::{LabelString, RegSets, Register, With};

use super::{CfgNode, RegisterSet};

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    uuid: Uuid,

    /// Labels for the entry point of this function
    labels: HashSet<With<LabelString>>,

    /// List of all nodes in the function. May not be in any particular order.
    nodes: RefCell<Vec<Rc<CfgNode>>>,

    /// Entry node of the function.
    entry: Rc<CfgNode>,

    /// Exit node of the function. Multiple exit points will be converted to a
    /// single exit point.
    exit: RefCell<Rc<CfgNode>>,

    /// The registers that are set ever in the function
    defs: RefCell<RegisterSet>,
}

impl Hash for Function {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
    }
}
impl Function {
    #[must_use]
    pub fn name(&self) -> LabelString {
        LabelString(
            self.entry
                .labels()
                .into_iter()
                .map(|x| x.data.0)
                .collect::<Vec<String>>()
                .join(", "),
        )
    }

    pub fn new(
        labels: Vec<With<LabelString>>,
        nodes: Vec<Rc<CfgNode>>,
        entry: Rc<CfgNode>,
        exit: Rc<CfgNode>,
    ) -> Self {
        Function {
            uuid: Uuid::new_v4(),
            labels: labels.into_iter().collect::<HashSet<_>>(),
            nodes: RefCell::new(nodes),
            entry,
            exit: RefCell::new(exit),
            defs: RefCell::new(RegisterSet::new()),
        }
    }

    #[must_use]
    pub fn labels(&self) -> HashSet<With<LabelString>> {
        self.entry.labels()
    }

    #[must_use]
    pub fn arguments(&self) -> RegisterSet {
        self.entry.live_out() & RegSets::argument()
    }

    #[must_use]
    pub fn returns(&self) -> RegisterSet {
        self.exit().live_in() & RegSets::ret()
    }

    /// Set the registers used by this function.
    #[must_use]
    pub fn set_defs(&self, defs: RegisterSet) -> bool {
        self.defs.replace_if_changed(defs)
    }

    /// Return the set of written registers.
    pub fn defs(&self) -> Ref<RegisterSet> {
        self.defs.borrow()
    }

    #[must_use]
    pub fn to_save(&self) -> RegisterSet {
        // Remove the stack pointer()
        (*self.defs() & RegSets::callee_saved()) - Register::X2
    }

    /// Set the instructions composing this function.
    #[must_use]
    pub fn set_nodes(&self, instructions: Vec<Rc<CfgNode>>) -> bool {
        self.nodes.replace_if_changed(instructions)
    }

    /// Return the instructions in the function.
    pub fn nodes(&self) -> Ref<Vec<Rc<CfgNode>>> {
        self.nodes.borrow()
    }

    /// Return the entry node of this function.
    pub fn entry(&self) -> Rc<CfgNode> {
        Rc::clone(&self.entry)
    }

    /// Return the exit node of this function. In general, this corresponds to a
    /// `ret` instruction.
    pub fn exit(&self) -> Ref<Rc<CfgNode>> {
        self.exit.borrow()
    }

    /// Set the exit node of this function.
    #[must_use]
    pub fn set_exit(&self, node: Rc<CfgNode>) -> bool {
        self.exit.replace_if_changed(node)
    }
}
