use std::{collections::HashMap, rc::Rc, vec};

use crate::{
    cfg::{Cfg, Function},
    parser::Register,
    parser::{Info, JumpLinkType, LabelString, ParserNode, With},
    passes::{CFGError, DiagnosticLocation, GenerationPass},
};

pub struct FunctionMarkupPass;
impl GenerationPass for FunctionMarkupPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CFGError>> {
        let mut label_function_map: HashMap<
            crate::parser::With<crate::parser::LabelString>,
            Rc<Function>,
        > = HashMap::new();

        // PASS 1
        // --------------------
        // Graph traversal to find functions

        for node in cfg.into_iter() {
            if node.node().is_return() {
                // Walk backwards from return label to find function starts
                let mut walked = Vec::new();
                let mut queue = vec![Rc::clone(&node)];
                let mut found = Vec::new();

                // For all items in the queue
                'inner: while let Some(n) = queue.pop() {
                    walked.push(Rc::clone(&n));

                    // If we reach the program entry, there's an issue
                    if n.node().is_program_entry() {
                        return Err(Box::new(CFGError::NoLabelForReturn(node.node())));
                    }

                    // If we find a function entry, we're done
                    if n.node().is_function_entry() {
                        found.push(Rc::clone(&n));
                        continue 'inner;
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
                        node.node(),
                        found.iter().flat_map(|x| x.labels.clone()).collect(),
                    )));
                }

                // Otherwise, we found a function and all its nodes
                if let Some(entry_node) = found.first() {
                    // Add the function to the map
                    let func = Rc::new(Function::new(
                        walked,
                        Rc::clone(entry_node),
                        Rc::clone(&node),
                    ));

                    // The label function map can have multiple entries corresponding to the single
                    // function, because that function has multiple labels. That makes this a bit
                    // messy because we want to ensure that every case is covered.
                    let mut exists = None;
                    'inn: for label in entry_node.labels() {
                        if let Some(existing_func) = label_function_map.get(&label) {
                            // Since we already have a "return" for this function,
                            // we will convert the new return to an unconditional jump
                            // to the existing return
                            exists = Some(existing_func);
                            break 'inn;
                        }
                    }

                    if let Some(existing_func) = exists {
                        // Convert the return node to an unconditional jump

                        // Get the return node, which will become an unconditional jump
                        let return_node = Rc::clone(&node);

                        // Get the existing return node -- will stay the same
                        let existing_return_node = Rc::clone(&existing_func.exit);

                        // Clear nexts of return node, and add existing return
                        // TODO convert next/prev setting to function to enforce
                        // At this point, the nexts of the return nodes should be all empty
                        // TODO assert that the nexts for both don't exist
                        return_node.clear_nexts();
                        return_node.insert_next(Rc::clone(&existing_return_node));

                        // Set return node's prev to original return node
                        existing_return_node.insert_prev(Rc::clone(&return_node));

                        // Convert node to jump
                        let inf = Info {
                            token: crate::parser::Token::Symbol("return".to_string()),
                            pos: return_node.node().range().clone(),
                            file: return_node.node().file(),
                        };

                        let inst = With::new(JumpLinkType::Jal, inf.clone());
                        let rd = With::new(Register::X0, inf.clone());
                        let name = With::new(LabelString("__return__".to_string()), inf.clone());
                        let new_node = ParserNode::new_jump_link(
                            inst,
                            rd,
                            name,
                            existing_return_node.node().token(),
                        );
                        return_node.set_node(new_node);
                    } else {
                        for label in entry_node.labels() {
                            label_function_map.insert(label.clone(), Rc::clone(&func));
                        }
                    }

                    // Add the function to the nodes
                    for func_node in &func.nodes {
                        func_node.set_function(Rc::clone(&func));
                    }
                } else {
                    // If we found no function entries, we have a problem
                    return Err(Box::new(CFGError::NoLabelForReturn(node.node())));
                }
            }
        }
        cfg.label_function_map = label_function_map;
        Ok(())
    }
}
