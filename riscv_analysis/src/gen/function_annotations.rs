use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    vec,
};

use crate::{
    cfg::{CfgNode, Cfg, Function},
    parser::{Info, JumpLinkType, LabelString, ParserNode, Register, With},
    passes::{CfgError, DiagnosticLocation, GenerationPass},
};

struct MarkData {
    pub found: HashSet<Register>,
    pub instructions: Vec<Rc<CfgNode>>,
    pub returns: Rc<CfgNode>,
}

pub struct FunctionMarkupPass;

impl FunctionMarkupPass {
    fn mark_reachable(entry: &Rc<CfgNode>, func: &Rc<Function>)
                      -> Result<MarkData, Box<CfgError>> {
        // Initialize data for graph search
        let mut queue = vec![Rc::clone(entry)];
        #[allow(clippy::mutable_key_type)]  // This is okay because we don't modify the field that is hashed
        let mut seen = HashSet::new();      // CFG nodes seen during the traversal
        let mut defs = HashSet::new();      // Registers this function writes to
        let mut returns = None;             // Return instructions in this function

        // Traverse the CFG for all nodes reachable from the entry point
        while let Some(node) = queue.pop() {
            // Only process each node at most once
            if seen.contains(&node) {
                continue;
            }
            seen.insert(Rc::clone(&node));

            // Mark the node as being a part of the given function
            node.insert_function(Rc::clone(func));

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
                    returns = Some(Rc::clone(&node));
                }
            }

            // Add all successor nodes to the queue
            let successors = node.nexts();
            for suc in successors.iter() {
                queue.push(Rc::clone(suc));
            }
        }

        let instructions = seen.into_iter().collect();

        if let Some(ret) = returns {
            Ok(MarkData { found: defs, instructions, returns: ret })
        }

        // TODO: Handle functions with no return statements
        else {
            Err(Box::new(CfgError::UnexpectedError))
        }
    }
}

impl GenerationPass for FunctionMarkupPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>> {
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
                              .cloned()
                              .collect::<Vec<_>>();

            // Insert a new function into the CFG
            let func = Rc::new(Function::new(
                labels.clone(), vec![], Rc::clone(&entry), Rc::clone(&entry)
            ));

            for label in &labels {
                label_function_map.insert(label.clone(), Rc::clone(&func));
            }


            // Mark all CFG nodes that are reachable from this entry point
            // FIXME: What to do if there is more than one return
            match Self::mark_reachable(&entry, &Rc::clone(&func)) {
                Ok(data) => {
                    func.set_defs(data.found);
                    func.set_nodes(data.instructions);
                    func.set_exit(data.returns);
                },
                Err(e) => { return Err(e); }
            }
        }

        cfg.label_function_map = label_function_map;
        Ok(())
    }
}
