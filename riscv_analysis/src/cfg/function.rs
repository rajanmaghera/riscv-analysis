use std::{collections::HashSet, rc::Rc, cell::RefCell};

use crate::{
    analysis::CustomClonedSets,
    parser::{LabelString, RegSets, Register, With, Info},
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
    /// Multiple exit points if there are any.
    ///
    /// To handle stack fixes where there are multiple
    /// exit points, we keep track of all the exits. This
    /// is not *usually* used for analysis but could be.
    pub other_exits: RefCell<HashSet<Rc<CFGNode>>>,
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
        other_exits: RefCell<HashSet<Rc<CFGNode>>>
    ) -> Self {
        Function {
            nodes,
            entry,
            exit,
            defs,
            other_exits,
        }
    }

    #[must_use]
    pub fn labels(&self) -> HashSet<With<LabelString>> {
        self.entry.labels()
    }

    #[must_use]
    pub fn arguments(&self) -> HashSet<Register> {
        self.entry.live_in().intersection_c(&RegSets::argument())
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

    pub fn add_other_exit(&self, other_exit: Rc<CFGNode>) {
        let mut i = self.other_exits.borrow_mut();
        i.insert(other_exit);

    }

    #[must_use]
    pub fn get_empty_label(&self) -> String {
        // get the first label
        let labels = self.labels();
        let label = labels.iter().next();
        let prefix = match label {
            Some(x) => {
                x.data.0.clone()
            }
            None => {
                "Func".into()
            }
        };

        let mut i = 1;
        while let Some(_) = labels.get(&With::new(LabelString(format!("{}__exit{}", prefix, i)), Info::default())) {
            i += 1;
        };

        format!("{}__exit{}", prefix, i)
    }
}
