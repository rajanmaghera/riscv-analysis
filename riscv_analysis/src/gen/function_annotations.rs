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
    pub instructions: Vec<Rc<CFGNode>>,
    pub returns: HashSet<Rc<CFGNode>>,
}

pub struct FunctionMarkupPass;

impl FunctionMarkupPass {
    fn mark_reachable(entry: Rc<CFGNode>, func: Rc<Function>)
                      -> Result<MarkData, Box<CFGError>> {
        // Initialize data for graph search
        let mut queue = vec![entry.clone()];
        let mut seen = HashSet::new();      // CFG nodes seen during the traversal
        let mut defs = HashSet::new();      // Registers this function writes to
        let mut returns = HashSet::new();   // Return instructions in this function

        // Traverse the CFG for all nodes reachable from the entry point
        while let Some(node) = queue.pop() {
            // Only process each node at most once
            if seen.contains(&node) {
                continue;
            } else {
                seen.insert(node.clone());
            }

            // Mark the node as being a part of the given function
            node.insert_function(func.clone());

            // Collect any registers written to by the node
            if let Some(dest) = node.node().stores_to() {
                defs.insert(dest.data);
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

        let instructions = seen.into_iter().collect();
        return Ok(MarkData {
            found: defs,
            instructions,
            returns,
        });
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

            // Get the labels for the entry block
            let labels = entry.labels()
                              .iter()
                              .map(|l| l.clone())
                              .collect::<Vec<_>>();

            // Insert a new function into the CFG
            let func = Rc::new(Function::new(
                labels.clone(),
                // FIXME: Dummy values
                vec![], entry.clone(), entry.clone()
            ));

            for label in labels.iter() {
                label_function_map.insert(label.clone(), func.clone());
            }


            // Mark all CFG nodes that are reachable from this entry point
            // FIXME: What to do if there is more than one return
            match Self::mark_reachable(entry, func.clone()) {
                Ok(data) => {
                    func.insert_defs(data.found);
                    func.insert_nodes(data.instructions);
                },
                Err(e) => { return Err(e); }
            }
        }

        cfg.label_function_map = label_function_map;
        Ok(())
    }
}
