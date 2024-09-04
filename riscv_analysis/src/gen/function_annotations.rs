use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    vec,
};

use crate::{
    cfg::{CFGNode, Cfg, Function},
    parser::{Info, JumpLinkType, LabelString, ParserNode, Register, With},
    passes::{CFGError, DiagnosticLocation, GenerationPass},
};

struct MarkData {
    pub found: HashSet<Register>,
    pub instructions: Vec<Rc<CFGNode>>,
    pub returns: Rc<CFGNode>,
}

pub struct FunctionMarkupPass;

impl FunctionMarkupPass {
    fn mark_reachable(entry: Rc<CFGNode>, func: Rc<Function>)
                      -> Result<MarkData, Box<CFGError>> {
        // Initialize data for graph search
        let mut queue = vec![entry.clone()];
        let mut seen = HashSet::new();      // CFG nodes seen during the traversal
        let mut defs = HashSet::new();      // Registers this function writes to
        let mut returns = None;             // Return instructions in this function

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
                // Set the newly found return to be an jump to the previously
                // found return.
                if let Some(ref prev_ret) = returns {
                    let found_ret = Rc::clone(&node);

                    // Fix the prevs & nexts of both returns
                    found_ret.clear_nexts();
                    found_ret.insert_next(Rc::clone(prev_ret));
                    prev_ret.insert_prev(Rc::clone(&found_ret));

                    // Convert the found return into a jump
                    let info = Info {
                        token: crate::parser::Token::Symbol("return".to_string()),
                        pos: found_ret.node().range().clone(),
                        file: found_ret.node().file(),
                    };

                    let inst = With::new(JumpLinkType::Jal, info.clone());
                    let rd = With::new(Register::X0, info.clone());
                    let name = With::new(LabelString("__return__".to_string()), info.clone());
                    let new_node = ParserNode::new_jump_link(
                        inst,
                        rd,
                        name,
                        prev_ret.node().token(),
                    );
                    found_ret.set_node(new_node);
                }

                // If this is the first return node, save it
                else {
                    returns = Some(node.clone());
                }
            }

            // Add all successor nodes to the queue
            let successors = node.nexts();
            for suc in successors.iter() {
                queue.push(suc.clone());
            }
        }

        let instructions = seen.into_iter().collect();

        // TODO: Handle functions with no return statements
        if let Some(ret) = returns {
            return Ok(MarkData { found: defs, instructions, returns: ret });
        } else {
            unimplemented!();
        }
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
                labels.clone(), vec![], entry.clone(), entry.clone()
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
                    func.set_exit(data.returns);
                },
                Err(e) => { return Err(e); }
            }
        }

        cfg.label_function_map = label_function_map;
        Ok(())
    }
}
