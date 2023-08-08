use std::{collections::HashSet, rc::Rc};

use crate::{
    analysis::CustomClonedSets,
    parser::{LabelString, RegSets, Register, With},
};

use super::CFGNode;

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    pub nodes: Vec<Rc<CFGNode>>,
    pub entry: Rc<CFGNode>, // Is only a FuncEntry node
    pub exit: Rc<CFGNode>,  // Multiple exit points will be converted to
                            // a single exit point
}

impl Function {
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
