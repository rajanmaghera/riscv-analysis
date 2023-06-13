use crate::cfg::directional::UseDefItems;
use crate::cfg::{ToRegBitmap, ToRegHashset};
use crate::parser::ast::ASTNode;
use crate::parser::register::Register;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::DirectionalWrapper;

pub struct LiveAnalysisResult {
    pub live_in: Vec<HashSet<Register>>,
    pub live_out: Vec<HashSet<Register>>,
    pub uncond_defs: Vec<HashSet<Register>>,
}

impl DirectionalWrapper<'_> {
    pub fn node_nexts(&self) -> HashMap<Rc<ASTNode>, HashSet<Rc<ASTNode>>> {
        let mut nexts = HashMap::new();
        for block in &self.cfg.blocks {
            let len = block.0.len();
            for (i, node) in block.0.iter().enumerate() {
                // determine next of each node
                let mut set = HashSet::new();
                if i == len - 1 {
                    let block = self.directions.get(block).unwrap().next.clone();
                    for next in block {
                        set.insert(next.0.first().unwrap().clone());
                    }
                } else {
                    set.insert(block.0[i + 1].clone());
                }
                nexts.insert(node.clone(), set);
            }
        }
        nexts
    }

    /* The magic of this app */
    pub fn live_analysis(&self) -> LiveAnalysisResult {
        #[derive(Clone)]
        struct LiveAnalysisNodeData {
            node: Rc<ASTNode>,
            kill: u32,
            gen: u32,
            live_in: u32,
            live_out: u32,
            u_def: u32,
            nexts: HashSet<Rc<ASTNode>>,
            prevs: HashSet<Rc<ASTNode>>,
        }

        let mut nodes = Vec::new();
        let mut astidx = HashMap::new();
        let mut funcidx = HashMap::new(); // index of start of function

        let mut idx = 0;
        for block in &self.cfg.blocks {
            for node in block.0.iter() {
                nodes.push(LiveAnalysisNodeData {
                    node: node.clone(),
                    kill: node.defs().to_bitmap(),
                    gen: node.uses().to_bitmap(),
                    live_in: 0,
                    live_out: 0,
                    u_def: 0,
                    nexts: self.next_ast_map.get(node).unwrap().clone(),
                    prevs: self.prev_ast_map.get(node).unwrap().clone(),
                });

                astidx.insert(node.clone(), idx);
                if let ASTNode::FuncEntry(entry) = node.borrow() {
                    funcidx.insert(entry.name.data.clone(), idx);
                }
                idx += 1;
            }
        }

        let mut changed = true;
        while changed {
            changed = false;
            for i in (0..nodes.len()).rev() {
                let mut node = nodes.get(i).unwrap().clone();

                let mut live_out = 0;
                for next in &node.nexts {
                    let idx = astidx.get(next).unwrap();
                    live_out |= nodes.get(*idx).unwrap().live_in;
                }

                node.live_out = live_out;

                let live_in_old = node.live_in;
                let u_def_old = node.u_def;

                // if call to a function
                if let Some(name) = node.node.call_name() {
                    // TODO ensure we are mutating values correctly
                    let func_entry = self.label_entry_map.get(&name.data).unwrap();
                    let func_return = self
                        .label_return_map
                        .get(&name.data)
                        .unwrap()
                        .iter()
                        .next()
                        .unwrap();
                    let func_entry_idx = astidx.get(&func_entry.clone()).unwrap();
                    let func_entry_data = nodes.get(*func_entry_idx).unwrap().clone();

                    let func_return_idx = astidx.get(&func_return.clone()).unwrap();
                    let func_return_data = nodes.get_mut(*func_return_idx).unwrap();

                    node.u_def = node.live_out;
                    node.live_in = func_entry_data.live_in
                        | (node.live_out & !super::caller_saved_registers());
                    func_return_data.live_in =
                        func_return_data.live_in | (node.live_out & func_return_data.u_def);

                // else if return from a function
                // aka. else if node is value in label_return_map
                } else if self
                    .label_return_map
                    .values()
                    .find(|x| x.iter().next().unwrap().clone() == node.node)
                    .is_some()
                {
                    // AND all the unconditional defs of the previous nodes
                    let mut new_u_def = u32::MAX;
                    if node.prevs.len() == 0 {
                        new_u_def = 0;
                    } else {
                        for prev in node.prevs.clone() {
                            let idx = astidx.get(&prev).unwrap();
                            new_u_def &= nodes[*idx].u_def;
                        }
                    }
                    node.u_def = new_u_def;
                } else {
                    let mut new_u_def = u32::MAX;
                    if node.prevs.len() == 0 {
                        new_u_def = 0;
                    } else {
                        for prev in node.prevs.clone() {
                            let idx = astidx.get(&prev).unwrap();
                            new_u_def &= nodes[*idx].u_def;
                        }
                    }

                    node.u_def = new_u_def | node.kill;
                    node.live_in = node.gen | (node.live_out & !node.kill);
                }

                if live_in_old != node.live_in || u_def_old != node.u_def {
                    changed = true;
                }

                let node_ref = nodes.get_mut(i).unwrap();
                *node_ref = node;
            }
        }

        let mut live_in = Vec::new();
        let mut live_out = Vec::new();
        let mut uncond_defs = Vec::new();
        nodes.iter().for_each(|node| {
            live_in.push(node.live_in.to_hashset());
            live_out.push(node.live_out.to_hashset());
            uncond_defs.push(node.u_def.to_hashset());
        });
        LiveAnalysisResult {
            live_in,
            live_out,
            uncond_defs,
        }
    }

    // // TODO deprecate
    // pub fn calculate_in_out(&self) -> (Vec<HashSet<Register>>, Vec<HashSet<Register>>) {
    //     let mut defs = Vec::new();
    //     let mut uses = Vec::new();
    //     let mut ins = Vec::new();
    //     let mut outs = Vec::new();
    //     let mut nexts = Vec::new();
    //     let mut ast = Vec::new();
    //     let mut astidx = HashMap::new();
    //     let mut funcidx = HashMap::new(); // index of start of function

    //     let mut big_idx = 0;
    //     for block in &self.cfg.blocks {
    //         for node in block.0.iter() {
    //             // TODO ensure basic block cannot be empty
    //             ast.push(node.clone());
    //             astidx.insert(node.clone(), big_idx);
    //             nexts.push(self.next_ast_map.get(node).unwrap().clone());
    //             uses.push(node.uses().to_bitmap());
    //             defs.push(node.defs().to_bitmap());
    //             ins.push(0);
    //             outs.push(0);

    //             if let ASTNode::FuncEntry(entry) = node.borrow() {
    //                 funcidx.insert(entry.name.data.clone(), big_idx);
    //             }

    //             big_idx += 1;
    //         }
    //     }

    //     // INTER-PROCEDURAL ANALYSIS SETUP
    //     let mut func_call_idx = Vec::new();
    //     let mut rets = Vec::new();
    //     for block in &self.cfg.blocks {
    //         for node in block.0.iter() {
    //             if let Some(name) = node.call_name() {
    //                 func_call_idx.push(Some(funcidx.get(&name.data).unwrap().clone()));
    //                 let mut retset = HashSet::new();
    //                 let labels = self.label_return_map.get(&name.data).unwrap().clone();
    //                 for ret in labels {
    //                     let id = astidx.get(&ret).unwrap().clone();
    //                     retset.insert(id);
    //                 }
    //                 rets.push(retset);
    //             } else {
    //                 func_call_idx.push(None);
    //                 rets.push(HashSet::new());
    //             }
    //         }
    //     }

    //     let garbage_values = vec![
    //         Register::X1,
    //         Register::X10,
    //         Register::X11,
    //         Register::X12,
    //         Register::X13,
    //         Register::X14,
    //         Register::X15,
    //         Register::X16,
    //         Register::X17,
    //         Register::X5,
    //         Register::X6,
    //         Register::X7,
    //         Register::X28,
    //         Register::X29,
    //         Register::X30,
    //         Register::X31,
    //     ]
    //     .into_iter()
    //     .collect::<HashSet<_>>()
    //     .to_bitmap();

    //     // calculate the in and out registers for every statement
    //     // let mut rounds = 0;
    //     // while rounds < 3 {
    //     let mut changed = true;
    //     while changed {
    //         changed = false;
    //         let len = defs.len();
    //         for i in (0..len).rev() {
    //             // get union of IN of all successors of this node
    //             let mut out = 0;
    //             for next in &nexts[i] {
    //                 let idx = astidx.get(next).unwrap();
    //                 out |= ins[*idx].clone();
    //             }
    //             outs[i] = out;

    //             // if this is a call to a function, set the use of this
    //             // block to the IN of the function
    //             if let Some(idx) = func_call_idx[i] {
    //                 uses[i] = ins[idx].clone();
    //                 // defs of this are all garbage values
    //                 defs[i] = garbage_values.clone();
    //                 for ret in rets[i].clone() {
    //                     // if this is a return statement, set the use of this
    //                     // block to the IN of the function

    //                     // calculate unconditional defs
    //                     let mut nodes = Vec::new();
    //                     let mut nodeidx = HashMap::new();
    //                     let mut prevs = Vec::new();
    //                     let mut always_defs = Vec::new();
    //                     let mut visited = HashSet::new();
    //                     let mut worklist = vec![ast[ret].clone()];
    //                     let mut new_idx = 0;
    //                     while let Some(node) = worklist.pop() {
    //                         if visited.contains(&node) {
    //                             continue;
    //                         }
    //                         visited.insert(node.clone());
    //                         nodes.push(node.clone());
    //                         prevs.push(self.prev_ast_map.get(&node).unwrap().clone());
    //                         worklist.extend(self.prev_ast_map.get(&node).unwrap().clone());
    //                         nodeidx.insert(node.clone(), new_idx);
    //                         always_defs.push(if node.is_call() {
    //                             let id = astidx.get(&node).unwrap().clone();
    //                             outs[id].clone()
    //                         } else {
    //                             node.defs().to_bitmap()
    //                         });
    //                         new_idx += 1;
    //                     }
    //                     let mut inner_changed = true;
    //                     while inner_changed == true {
    //                         inner_changed = false;
    //                         for i in 0..nodes.len() {
    //                             let old_always_def = always_defs[i].clone();
    //                             let new_or = if prevs[i].len() == 0 {
    //                                 0
    //                             } else {
    //                                 let mut or = u32::MAX;
    //                                 for prev in prevs[i].clone() {
    //                                     let idx = nodeidx.get(&prev).unwrap();
    //                                     or &= always_defs[*idx].clone()
    //                                 }
    //                                 or
    //                             };
    //                             always_defs[i] |= new_or;
    //                             if old_always_def != always_defs[i] {
    //                                 inner_changed = true;
    //                             }
    //                         }
    //                     }

    //                     uses[ret] |= out.clone() & always_defs[0].clone();
    //                 }
    //             }

    //             // if this is a call to a function, set the

    //             // calculate new IN
    //             let in_old = ins[i].clone();
    //             ins[i] = uses[i].clone() | (outs[i].clone() & !defs[i].clone());
    //             if in_old != ins[i] {
    //                 changed = true;
    //             }
    //         }
    //     }

    //     // ARGUMENT GUESSING
    //     //

    //     // convert the in and out registers to hashsets
    //     let mut ins_hashset = Vec::new();
    //     let mut outs_hashset = Vec::new();
    //     for i in 0..ins.len() {
    //         ins_hashset.push(ins[i].to_hashset());
    //         outs_hashset.push(outs[i].to_hashset());
    //     }

    //     // print the in and out registers
    //     let mut i = 0;
    //     for (ii, block) in self.cfg.blocks.iter().enumerate() {
    //         println!("BLOCK: {}", ii);
    //         for (_, node) in block.0.iter().enumerate() {
    //             println!(
    //                 "IN: {:?}, OUT: {:?}, USES: {:?}, DEFS: {:?}",
    //                 ins_hashset[i],
    //                 outs_hashset[i],
    //                 node.uses(),
    //                 node.defs()
    //             );
    //             i += 1;
    //         }
    //     }
    //     (ins_hashset, outs_hashset)
    // }
}
