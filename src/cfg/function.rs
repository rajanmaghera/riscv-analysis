use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    vec,
};

use crate::{
    analysis::CustomClonedSets,
    parser::{LabelString, Node, RegSets, Register, With},
    passes::{CFGError, GenerationPass},
};

use super::{BaseCFG, CFGNode};

// TODO assert FuncEntry nodes are only in the entry spot
// TODO assert that all Functions correspond to one FuncEntry/FuncExit node
// TODO do we need nodes?
#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub nodes: Vec<Rc<CFGNode>>,
    pub entry: Rc<CFGNode>, // Is only a FuncEntry node
    pub exit: Rc<CFGNode>,  // Multiple exit points will be converted to
                            // a single exit point
}

impl Function {
    pub fn new(nodes: Vec<Rc<CFGNode>>, entry: Rc<CFGNode>, exit: Rc<CFGNode>) -> Self {
        Function { nodes, entry, exit }
    }

    #[inline(always)]
    pub fn labels(&self) -> HashSet<With<LabelString>> {
        self.entry.labels()
    }

    pub fn arguments(&self) -> HashSet<Register> {
        self.entry.live_in().intersection_c(&RegSets::argument())
    }

    pub fn returns(&self) -> HashSet<Register> {
        self.exit.live_in().intersection_c(&RegSets::ret())
    }
}