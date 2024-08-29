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

pub struct FunctionMarkupPass;

impl FunctionMarkupPass {
    fn mark_reachable(entry: Rc<CFGNode>, func: Rc<Function>)
                      -> Result<HashSet<Register>, Box<CFGError>> {
        // Initialize data for graph search
        let mut queue = vec![entry.clone()];
        let mut seen = HashSet::new();

        // Which registers does this function write to?
        let mut found = HashSet::new();

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
            if node.function().is_some() {
                return Err(Box::new(
                    CFGError::MultipleFunctions(node.node(), entry.labels())
                ))
            }

            // Mark this node as being a part of this function
            node.set_function(func.clone());

            // Collect any registers written to by this instruction
            if let Some(dest) = node.node().stores_to() {
                found.insert(dest.data);
            }

            // Add all successor nodes to the queue
            let successors = node.nexts();
            for suc in successors.iter() {
                queue.push(suc.clone());
            }
        }

        return Ok(found);
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
                Ok(defs) => { func.insert_defs(defs); },
                Err(e) => { return Err(e); }
            }
        }

        cfg.label_function_map = label_function_map;
        Ok(())
    }
}
