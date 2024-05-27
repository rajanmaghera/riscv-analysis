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
    /// The registers that are set ever in the function
    pub defs: HashSet<Register>,
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
        nodes: Vec<Rc<CFGNode>>,
        entry: Rc<CFGNode>,
        exit: Rc<CFGNode>,
        defs: HashSet<Register>,
    ) -> Self {
        Function {
            nodes,
            entry,
            exit,
            defs,
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
        self.exit.live_in().intersection_c(&RegSets::ret())
    }

    #[must_use]
    pub fn to_save(&self) -> HashSet<Register> {
        self.defs
            .intersection_c(&RegSets::callee_saved())
            // remove sp
            .difference_c(&vec![Register::X2].into_iter().collect())
    }
}
