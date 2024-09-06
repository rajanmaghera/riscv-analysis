use std::{cell::{Ref, RefCell}, collections::HashSet, hash::Hash, rc::Rc};

use crate::{
    analysis::CustomClonedSets,
    parser::{LabelString, RegSets, Register, With},
};

use super::CfgNode;

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    /// Labels for the entry point of this function
    labels: HashSet<With<LabelString>>,

    /// List of all nodes in the function. May not be in any particular order.
    nodes: RefCell<Vec<Rc<CfgNode>>>,

    /// Entry node of the function.
    pub entry: Rc<CfgNode>,

    /// Exit node of the function. Multiple exit points will be converted to a
    /// single exit point.
    exit: RefCell<Rc<CfgNode>>,

    /// The registers that are set ever in the function
    defs: RefCell<HashSet<Register>>,
}

impl Hash for Function {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name().hash(state);
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
            labels: labels.into_iter().collect::<HashSet<_>>(),
            nodes: RefCell::new(nodes),
            entry,
            exit: RefCell::new(exit),
            defs: RefCell::new(HashSet::new()),
        }
    }

    #[must_use]
    pub fn labels(&self) -> HashSet<With<LabelString>> {
        self.entry.labels()
    }

    #[must_use]
    pub fn arguments(&self) -> HashSet<Register> {
        self.entry.live_out().intersection_c(&RegSets::argument())
    }

    #[must_use]
    pub fn returns(&self) -> HashSet<Register> {
        self.exit().live_in().intersection_c(&RegSets::ret())
    }

    /// Insert the set of registers used by this function.
    pub fn insert_defs(&self, defs: HashSet<Register>) {
        *self.defs.borrow_mut() = defs;
    }

    /// Return the set of written registers.
    pub fn defs(&self) -> Ref<HashSet<Register>> {
        self.defs.borrow()
    }

    #[must_use]
    pub fn to_save(&self) -> HashSet<Register> {
        self.defs()
            .intersection_c(&RegSets::callee_saved())
            // remove sp
            .difference_c(&vec![Register::X2].into_iter().collect())
    }

    /// Insert the instructions composing this function.
    pub fn insert_nodes(&self, instructions: Vec<Rc<CfgNode>>) {
        *self.nodes.borrow_mut() = instructions;
    }

    /// Return the instructions in the function.
    pub fn nodes(&self) -> Ref<Vec<Rc<CfgNode>>> {
        self.nodes.borrow()
    }

    /// Return the exit node of this function. In general, this corresponds to a
    /// `ret` instruction.
    pub fn exit(&self) -> Ref<Rc<CfgNode>> {
        self.exit.borrow()
    }

    /// Set the exit node of this function.
    pub fn set_exit(&self, node: Rc<CfgNode>) {
        *self.exit.borrow_mut() = node;
    }
}
