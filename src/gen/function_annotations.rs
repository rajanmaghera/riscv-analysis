use std::{collections::HashMap, rc::Rc, vec};

use crate::{
    cfg::{Function, CFG},
    passes::{CFGError, GenerationPass},
};

pub struct FunctionMarkupPass;
impl GenerationPass for FunctionMarkupPass {
    fn run(cfg: &mut CFG) -> Result<(), Box<CFGError>> {
        let mut label_function_map = HashMap::new();

        // PASS 1
        // --------------------
        // Graph traversal to find functions

        for node in cfg.into_iter() {
            if node.node.is_return() {
                // Walk backwards from return label to find function starts
                let mut walked = Vec::new();
                let mut queue = vec![Rc::clone(&node)];
                let mut found = Vec::new();

                // For all items in the queue
                'inn: while let Some(n) = queue.pop() {
                    walked.push(Rc::clone(&n));

                    // If we reach the program entry, there's an issue
                    if n.node.is_program_entry() {
                        return Err(Box::new(CFGError::NoLabelForReturn(node.node.clone())));
                    }

                    // If we find a function entry, we're done
                    if n.node.is_function_entry() {
                        found.push(Rc::clone(&n));
                        continue 'inn;
                    }

                    // Otherwise, add all previous nodes to the queue
                    for prev in n.prevs().iter() {
                        if !walked.contains(prev) {
                            queue.push(Rc::clone(prev));
                        }
                    }
                }

                // If we found multiple function entries, we have a problem
                if found.len() > 1 {
                    return Err(Box::new(CFGError::MultipleLabelsForReturn(
                        node.node.clone(),
                        found.iter().flat_map(|x| x.labels.clone()).collect(),
                    )));
                }

                // Otherwise, we found a function and all its nodes
                if let Some(entry_node) = found.first() {
                    let func = Rc::new(Function::new(
                        walked,
                        Rc::clone(entry_node),
                        Rc::clone(&node),
                    ));

                    // Add the function to the map
                    for label in func.labels() {
                        if let Some(func2) =
                            label_function_map.insert(label.clone(), Rc::clone(&func))
                        {
                            // If we already have a function for this label, we have a problem
                            let mut labels = func2.labels();
                            labels.extend(func.labels());
                            return Err(Box::new(CFGError::MultipleReturnsForLabel(
                                labels.into_iter().collect(),
                                vec![func.exit.node.clone(), func2.exit.node.clone()]
                                    .into_iter()
                                    .collect(),
                            )));
                        }
                    }

                    // Add the function to the nodes
                    for node in &func.nodes {
                        node.set_function(Rc::clone(&func));
                    }
                } else {
                    // If we found no function entries, we have a problem
                    return Err(Box::new(CFGError::NoLabelForReturn(node.node.clone())));
                }
            }
        }
        cfg.label_function_map = label_function_map;
        Ok(())
    }
}
