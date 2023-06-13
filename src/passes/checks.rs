use crate::cfg::{AnnotatedCFG, UseDefItems};
use crate::cfg::{DirectionalCFG, CFG};
use crate::parser::register::Register;
use crate::parser::token::LineDisplay;
use std::collections::{HashMap, HashSet, VecDeque};

use super::*;

// Checks are passes that occur after the CFG is built. As much data as possible is collected
// during the CFG build. Then, the data is applied via a check.

pub struct SaveToZeroCheck;
impl Pass for SaveToZeroCheck {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let mut errors = Vec::new();
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                if let Some(register) = (*node).stores_to() {
                    if register == Register::X0 {
                        errors.push(PassError::SaveToZero(register.get_range()));
                    }
                }
            }
        }

        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}

pub struct DeadValueCheck;
impl Pass for DeadValueCheck {
    fn run(&self, cfg: &AnnotatedCFG) -> Result<(), PassErrors> {
        let node_next = &cfg.next_ast_map;
        let live_data = &cfg.liveness;
        let mut errors = Vec::new();
        let mut i: usize = 0;
        // recalc mapping of nodes to idx
        let mut node_idx = HashMap::new();
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                node_idx.insert(node, i);
                i += 1;
            }
        }
        let mut i = 0;
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                // check for any assignments that don't make it
                // to the end of the node
                for def in node.defs() {
                    if !live_data.live_out.get(i).unwrap().contains(&def) {
                        errors.push(PassError::DeadAssignment(node.get_range().clone()));
                    }
                }

                // check the out of the node for any uses that
                // should not be there (temporaries)
                if let Some(name) = node.call_name() {
                    // check the expected return values of the function:
                    // get IN of return statement of function
                    let ret_state = cfg.label_return_map.get(&name.data).unwrap().clone();
                    // get first element from hashset
                    let ret_node = ret_state.iter().next().unwrap();
                    // get index of return node
                    let ret_idx = node_idx.get(ret_node).unwrap().clone();
                    // get IN of return node
                    let ret_in = live_data.live_in.get(ret_idx).unwrap().clone();
                    // subtract the IN from garbage registers
                    let out: HashSet<Register> =
                        Register::garbages().difference(&ret_in).cloned().collect();

                    // if there is anything left, then there is an error
                    // for each item, keep going to the next node until a use of
                    // that item is found
                    for item in out {
                        let mut queue = VecDeque::new();
                        // push the next nodes onto the queue
                        for next in node_next.get(&node).unwrap() {
                            queue.push_back(next.clone());
                        }

                        // keep track of visited nodes
                        let mut visited = HashSet::new();
                        visited.insert(node.clone());

                        // visit each node in the queue
                        // if the error is found, add error
                        // if not, add the next nodes to the queue
                        while let Some(next) = queue.pop_front() {
                            if visited.contains(&next) {
                                continue;
                            }
                            visited.insert(next.clone());
                            if next.uses().contains(&item) {
                                // find the use
                                let regs = next.uses_reg();
                                let mut it = None;
                                for reg in regs {
                                    if reg == item {
                                        it = Some(reg);
                                        break;
                                    }
                                }
                                if let Some(reg) = it {
                                    errors.push(PassError::InvalidUseAfterCall(
                                        reg.get_range().clone(),
                                        name.clone(),
                                    ));
                                }
                                break;
                            }
                            for next_next in node_next.get(&next).unwrap() {
                                queue.push_back(next_next.clone());
                            }
                        }
                    }
                }
                i += 1;
            }
        }
        if errors.len() > 0 {
            Err(PassErrors { errors })
        } else {
            Ok(())
        }
    }
}
