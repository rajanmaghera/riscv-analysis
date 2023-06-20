use crate::cfg::regset::RegSets;
use crate::cfg::{ecall_in_outs, is_ecall_exit, AvailableValue, ToRegBitmap, ToRegHashset};
use crate::parser::ast::ASTNode;
use crate::parser::inst::BasicType;
use crate::parser::register::Register;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::{AvailableValueResult, DirectionalWrapper};

#[derive(Clone)]
pub struct LiveAnalysisResult {
    pub live_in: Vec<HashSet<Register>>,
    pub live_out: Vec<HashSet<Register>>,
    pub uncond_defs: Vec<HashSet<Register>>,
}

impl DirectionalWrapper {
    /* The magic of this app */
    pub fn live_analysis(&mut self, avail: &AvailableValueResult) -> LiveAnalysisResult {
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
            for node in &block.0 {
                // HACK - if ecall is an exit call, then remove nexts
                if let ASTNode::Basic(basic) = &(**node) {
                    if basic.inst.data == BasicType::Ecall {
                        if let Some(call_val) = avail.avail_in.get(idx).unwrap().get(&Register::X17)
                        {
                            if let AvailableValue::Constant(call_num) = call_val {
                                if is_ecall_exit(*call_num) {
                                    // for all nexts, remove their prev counterparts
                                    for next in self.next_ast_map.get(node).unwrap().clone() {
                                        self.prev_ast_map.get_mut(&next).unwrap().remove(node);
                                    }
                                    self.next_ast_map.get_mut(node).unwrap().clear();
                                }
                            }
                        }
                    }
                }

                nodes.push(LiveAnalysisNodeData {
                    node: node.clone(),
                    kill: node.kill().to_bitmap(),
                    gen: node.gen().to_bitmap(),
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
                if let Some(name) = node.node.calls_func_to() {
                    // BUG FUNCTION RETURN VALUES ARE PART OF U_DEFS OF FUNCTION

                    // TODO ensure we are mutating values correctly

                    let mut new_u_def = u32::MAX;
                    if node.prevs.is_empty() {
                        new_u_def = 0;
                    } else {
                        for prev in node.prevs.clone() {
                            let idx = astidx.get(&prev).unwrap();
                            new_u_def &= nodes[*idx].u_def;
                        }
                    }

                    let func_entry = self.label_entry_map.get(&name.data).unwrap();
                    let func_return = self
                        .label_return_map
                        .get(&name.data)
                        .unwrap()
                        .iter()
                        .next()
                        .unwrap();

                    let func_entry_idx = astidx.get(func_entry).unwrap();
                    let func_entry_data = nodes.get(*func_entry_idx).unwrap().clone();

                    let func_return_idx = astidx.get(func_return).unwrap();
                    let func_return_data = nodes.get_mut(*func_return_idx).unwrap();

                    let old = func_return_data.live_in;
                    func_return_data.live_in = func_return_data.gen
                        | func_return_data.live_in
                        | (node.live_out & func_return_data.u_def);
                    if old != func_return_data.live_in {
                        changed = true;
                    }
                    // UDEF = (Union of all prevs - kill (caller saved)) | UDEF_f
                    // NOTE: we use the UDEF_f because the udefs are all "candidates"
                    // for returns. If one happens to be the return, we can be sure
                    // that it is always defined. Otherwise, it is an error becuase
                    // we don't know if it is defined or not, so we could be reading
                    // a garbage value.
                    // TLDR: udef -> return values are a safeguard that the value
                    // has to come from the function.
                    node.u_def =
                        (new_u_def & !RegSets::caller_saved().to_bitmap()) | func_return_data.u_def;
                    node.live_in = (func_entry_data.live_in & RegSets::argument().to_bitmap())
                        | (node.live_out & !RegSets::caller_saved().to_bitmap());

                    // else if ecall (similar logic to function call, but we don't
                    // need to markup inside a function
                } else if let ASTNode::Basic(_x) = node.node.borrow() {
                    // if we have access to a constant value for the ecall

                    node.u_def = node.live_out;
                    node.live_in = HashSet::from_iter(vec![Register::X17]).to_bitmap()
                        | (node.live_out & !RegSets::caller_saved().to_bitmap());

                    if let Some(call_val) = avail.avail_in.get(i).unwrap().get(&Register::X17) {
                        if let AvailableValue::Constant(call_num) = call_val {
                            if let Some((args, _rets)) = ecall_in_outs(*call_num) {
                                // TODO do something about return values?
                                node.live_in |= args.to_bitmap();
                            }
                        }
                    }

                // else if return from a function
                // aka. else if node is value in label_return_map
                } else if self
                    .label_return_map
                    .values()
                    .any(|x| x.iter().next().unwrap().clone() == node.node)
                {
                    // AND all the unconditional defs of the previous nodes
                    let mut new_u_def = u32::MAX;
                    if node.prevs.is_empty() {
                        new_u_def = 0;
                    } else {
                        for prev in node.prevs.clone() {
                            let idx = astidx.get(&prev).unwrap();
                            new_u_def &= nodes[*idx].u_def;
                        }
                    }
                    node.u_def = new_u_def;
                } else if let ASTNode::FuncEntry(_) = node.node.borrow() {
                    // if this is the entry of a function, then the unconditional
                    // defs are the IN of the function
                    node.live_in = node.gen | (node.live_out & !node.kill);
                    node.u_def = node.live_in;
                } else {
                    let mut new_u_def = u32::MAX;
                    if node.prevs.is_empty() {
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

                if live_in_old != node.live_in {
                    changed = true;
                }
                if u_def_old != node.u_def {
                    changed = true;
                }

                let node_ref = nodes.get_mut(i).unwrap();
                *node_ref = node;
            }
        }

        let mut live_in = Vec::new();
        let mut live_out = Vec::new();
        let mut uncond_defs = Vec::new();
        for node in &nodes {
            live_in.push(node.live_in.to_hashset());
            live_out.push(node.live_out.to_hashset());
            uncond_defs.push(node.u_def.to_hashset());
        }
        LiveAnalysisResult {
            live_in,
            live_out,
            uncond_defs,
        }
    }
}
