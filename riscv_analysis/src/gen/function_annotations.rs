use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    vec,
};

use crate::{
    cfg::{CFGNode, Cfg, Function},
    parser::Register,
    passes::{CFGError, GenerationPass},
};

struct MarkData {
    pub found: HashSet<Register>,
    pub returns: HashSet<Rc<CFGNode>>,
}

pub struct FunctionMarkupPass;

impl FunctionMarkupPass {
    fn mark_reachable(entry: Rc<CFGNode>, func: Rc<Function>)
                      -> Result<MarkData, Box<CFGError>> {
        // Initialize data for graph search
        let mut queue = vec![entry.clone()];
        let mut seen = HashSet::new();
        let mut found = HashSet::new();     // Registers this function writes to
        let mut returns = HashSet::new();   // Return instructions in this function

        // Traverse the CFG for all nodes reachable from the entry point
        while let Some(node) = queue.pop() {
            // Only process each node at most once
            if seen.contains(&node) {
                continue;
            } else {
                seen.insert(node.clone());
            }

            // If this node is already marked, then two (or more) functions
            // share some code
            if let Some(func) = node.function().as_deref() {
                let labels: HashSet<_>
                    = node.labels().union(&func.labels()).map(|e| e.clone()).collect();
                return Err(Box::new(
                    CFGError::MultipleFunctions(node.node(), labels)
                ));
            }

            // Mark the node as being a part of the given function
            node.set_function(func.clone());

            // Collect any registers written to by the node
            if let Some(dest) = node.node().stores_to() {
                found.insert(dest.data);
            }

            // Collect return instructions
            if node.node().is_return() {
                returns.insert(node.clone());
            }

            // Add all successor nodes to the queue
            let successors = node.nexts();
            for suc in successors.iter() {
                queue.push(suc.clone());
            }
        }

        // If there is more than one return, signal an error
        // TODO: Handle functions with more than one return statement
        if returns.len() != 1 {
            unimplemented!();
        }

        return Ok(MarkData { found, returns });
    }
}

impl GenerationPass for FunctionMarkupPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CFGError>> {
        let mut label_function_map: HashMap<
            crate::parser::With<crate::parser::LabelString>,
            Rc<Function>,
        > = HashMap::new();

        // PASS 1
        // --------------------
        // Graph traversal to find functions

        for entry in &*cfg {
            // Skip all nodes that are not entry points
            if !entry.node().is_function_entry() {
                continue;
            }

            // Insert a new function into the CFG
            let func = Rc::new(Function::new(
                // FIXME: Dummy values
                vec![], entry.clone(), entry.clone()
            ));

            let labels = entry.labels()
                              .iter()
                              .map(|l| l.clone())
                              .collect::<Vec<_>>();
            for label in labels.iter() {
                label_function_map.insert(label.clone(), func.clone());
            }

            // Mark all CFG nodes that are reachable from this entry point
            // FIXME: What to do if there is more than one return
            match Self::mark_reachable(entry, func.clone()) {
                Ok(data) => { func.insert_defs(data.found); },
                Err(e) => { return Err(e); }
            }
        }

        cfg.label_function_map = label_function_map;
        Ok(())
    }
}
