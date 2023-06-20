use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

use itertools::Itertools;

use crate::parser::{BasicType, Node, Register};

use super::{
    regset::RegSets, AvailableValue, AvailableValueResult, BasicBlock, DirectionMap,
    DirectionalWrapper, LabelToNode, LabelToNodes, LiveAnalysisResult, NodeToNodes,
    NodeToPotentialLabel, CFG,
};

// struct AnnotatedNode {
//     node: ASTNode,
//     live_in: HashSet<Register>,
//     live_out: HashSet<Register>,
//     u_def: HashSet<Register>,
//     values_in: HashMap<Register, AvailableValue>,
//     values_out: HashMap<Register, AvailableValue>,
//     stack_in: HashMap<i32, AvailableValue>,
//     stack_out: HashMap<i32, AvailableValue>,
//     function: RefCell<Option<Rc<AnnotatedFunction>>>,
//     next: RefCell<HashSet<Rc<AnnotatedNode>>>,
//     prev: RefCell<HashSet<Rc<AnnotatedNode>>>,
// }

// impl Hash for AnnotatedNode {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.node.hash(state);
//     }
// }
// impl PartialEq for AnnotatedNode {
//     fn eq(&self, other: &Self) -> bool {
//         self.node == other.node
//     }
// }
// impl Eq for AnnotatedNode {}

// impl AnnotatedNode {
//     fn next(&self) -> HashSet<Rc<AnnotatedNode>> {
//         self.next.borrow().clone()
//     }
//     fn prev(&self) -> HashSet<Rc<AnnotatedNode>> {
//         self.prev.borrow().clone()
//     }
//     fn function(&self) -> Option<Rc<AnnotatedFunction>> {
//         self.function.borrow().clone()
//     }
// }

// struct AnnotatedFunction {
//     id: uuid::Uuid,
//     nodes: Vec<Rc<AnnotatedNode>>,
//     names: HashSet<Label>,
//     entry: Rc<AnnotatedNode>,
//     exit: Rc<AnnotatedNode>,
//     called_from: HashSet<Rc<AnnotatedNode>>,
//     //
// }

// struct NewAnnotatedCFG {
//     nodes: Vec<Rc<AnnotatedNode>>,
//     functions: HashMap<LabelString, Rc<AnnotatedFunction>>,
//     //
// }

// impl AnnotatedCFG {
//     fn to_new(&self) -> NewAnnotatedCFG {
//         let mut nodes = Vec::new();
//         let mut old_new_map = HashMap::new();

//         for (i, orig_node) in self.nodes.clone().into_iter().enumerate() {
//             let node = AnnotatedNode {
//                 node: orig_node.as_ref().clone(),
//                 live_in: self.liveness.live_in[i].clone(),
//                 live_out: self.liveness.live_out[i].clone(),
//                 u_def: self.liveness.uncond_defs[i].clone(),
//                 values_in: self.available.avail_in[i].clone(),
//                 values_out: self.available.avail_out[i].clone(),
//                 stack_in: self.available.stack_in[i].clone(),
//                 stack_out: self.available.stack_out[i].clone(),
//                 function: RefCell::new(None),
//                 next: RefCell::new(HashSet::new()),
//                 prev: RefCell::new(HashSet::new()),
//             };
//             let new_node = Rc::new(node);
//             old_new_map.insert(orig_node.clone(), new_node.clone());
//             nodes.push(new_node);
//         }

//         for (i, new_node) in nodes.clone().into_iter().enumerate() {
//             let node = self.nodes[i].clone();
//             let nexts = self.next_ast_map.get(&node).unwrap();
//             let prevs = self.prev_ast_map.get(&node).unwrap();
//             for next in nexts {
//                 let next = old_new_map.get(next).unwrap();
//                 new_node.next.borrow_mut().insert(next.clone());
//                 next.prev.borrow_mut().insert(new_node.clone());
//             }
//             for prev in prevs {
//                 let prev = old_new_map.get(prev).unwrap();
//                 new_node.prev.borrow_mut().insert(prev.clone());
//                 prev.next.borrow_mut().insert(new_node.clone());
//             }
//         }

//         NewAnnotatedCFG {
//             nodes,
//             functions: HashMap::new(),
//         }
//     }
// }
// TODO annotation that tells every node what function it's in
#[derive(Clone)]
pub struct AnnotatedCFG {
    // TODO convert all maps from nodes/indices to fields on the node, so there's
    // no get nonsense
    // TODO convert each map that is from a function to a struct so that all of them
    // share the same struct from one call, same with direction
    pub blocks: Vec<Rc<BasicBlock>>,
    pub nodes: Vec<Rc<Node>>,
    pub labels: HashMap<String, Rc<BasicBlock>>,
    pub labels_for_branch: Vec<Vec<String>>,
    pub directions: DirectionMap,
    pub node_function_map: NodeToPotentialLabel,
    pub label_entry_map: LabelToNode,
    pub label_return_map: LabelToNodes,
    pub label_call_map: LabelToNodes,
    pub next_ast_map: NodeToNodes,
    pub prev_ast_map: NodeToNodes,
    pub liveness: LiveAnalysisResult,
    pub available: AvailableValueResult,
}

impl IntoIterator for AnnotatedCFG {
    type Item = Rc<Node>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl From<CFG> for AnnotatedCFG {
    fn from(cfg: CFG) -> Self {
        let dcfg = DirectionalWrapper::from(cfg);
        let awrap = AnalysisWrapper::from(dcfg);

        Self {
            liveness: awrap.liveness,
            available: awrap.available,
            directions: awrap.dcfg.directions,
            label_entry_map: awrap.dcfg.label_entry_map,
            label_return_map: awrap.dcfg.label_return_map,
            label_call_map: awrap.dcfg.label_call_map,
            next_ast_map: awrap.dcfg.next_ast_map,
            prev_ast_map: awrap.dcfg.prev_ast_map,
            node_function_map: awrap.dcfg.node_function_map,
            nodes: awrap.dcfg.cfg.nodes,
            blocks: awrap.dcfg.cfg.blocks,
            labels: awrap.dcfg.cfg.labels,
            labels_for_branch: awrap.dcfg.cfg.labels_for_branch,
        }
    }
}

pub struct AnalysisWrapper {
    pub dcfg: DirectionalWrapper,
    pub liveness: LiveAnalysisResult,
    pub available: AvailableValueResult,
}

impl From<DirectionalWrapper> for AnalysisWrapper {
    fn from(dwrap: DirectionalWrapper) -> Self {
        let mut dwrap = dwrap;
        let avail = dwrap.available_value_analysis();
        Self {
            liveness: dwrap.live_analysis(&avail),
            dcfg: dwrap,
            available: avail,
        }
    }
}

impl Display for AnnotatedCFG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut index = 0;

        let mut labels = self.labels_for_branch.iter();
        for block in &self.blocks {
            f.write_str("+---------\n")?;
            f.write_str(&format!(
                "| LABELS: {:?}, ID: {}\n",
                labels.next().unwrap(),
                &block.1.as_simple().to_string()[..8]
            ))?;
            f.write_str(&format!(
                "| PREV: [{}]\n",
                self.directions
                    .get(block)
                    .unwrap()
                    .prev
                    .iter()
                    .collect::<Vec<_>>()
                    .iter()
                    .map(|x| x.1.as_simple().to_string()[..8].to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))?;

            f.write_str("| ****\n")?;
            for node in &block.0 {
                f.write_str(&format!(
                    "| {:>3}: {}\n|  in: {:<20}\n| out: {:<20}\n",
                    index,
                    node,
                    self.liveness
                        .live_in
                        .get(index)
                        .unwrap()
                        .iter()
                        .sorted()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.liveness
                        .live_out
                        .get(index)
                        .unwrap()
                        .iter()
                        .sorted()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                ))?;
                f.write_str(&format!(
                    "| val: {}\n",
                    self.available
                        .avail_out
                        .get(index)
                        .unwrap()
                        .iter()
                        .sorted_by_key(|x| x.0)
                        .map(|(k, v)| format!("[{k}: {v}]"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))?;
                f.write_str(&format!(
                    "| stk: {}\n",
                    self.available
                        .stack_out
                        .get(index)
                        .unwrap()
                        .iter()
                        .sorted_by_key(|x| x.0)
                        .map(|(k, v)| format!("[{k}: {v}]"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))?;
                f.write_str(&format!(
                    "| udf: {}\n",
                    self.liveness
                        .uncond_defs
                        .get(index)
                        .unwrap()
                        .iter()
                        .sorted()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                ))?;
                index += 1;
            }
            f.write_str("+---------\n")?;
        }
        f.write_str("FUNCTION DATA:\n")?;
        for (k, _) in &self.label_entry_map {
            f.write_str(&format!(
                "{}: {} -> {}\n",
                k.0,
                self.function_args(&k.0)
                    .unwrap_or(HashSet::new())
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                self.function_rets(&k.0)
                    .unwrap_or(HashSet::new())
                    .into_iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))?;
        }
        Ok(())
    }
}

impl AnnotatedCFG {
    pub fn function_args(&self, name: &str) -> Option<HashSet<Register>> {
        let val = self
            .label_entry_map
            .get(&crate::parser::LabelString(name.to_owned()))?;
        let idx = self.nodes.iter().position(|x| x == val)?;
        let node = self.liveness.live_in.get(idx)?.clone();
        let node = node.intersection(&RegSets::argument()).copied().collect();
        Some(node)
    }

    pub fn function_rets(&self, name: &str) -> Option<HashSet<Register>> {
        let val = self
            .label_return_map
            .get(&crate::parser::LabelString(name.to_owned()))?
            .iter()
            .next()?;
        let idx = self.nodes.iter().position(|x| x == val)?;
        let node = self.liveness.live_in.get(idx)?.clone();
        let node = node.intersection(&RegSets::ret()).copied().collect();
        Some(node)
    }

    pub fn _is_program_exit(&self, node: &Rc<Node>) -> bool {
        match &*(*node) {
            Node::Basic(x) => {
                let idx = self.nodes.iter().position(|x| x == node).unwrap();

                let avail_a7 = self
                    .available
                    .avail_out
                    .get(idx)
                    .unwrap()
                    .get(&Register::X17)
                    .unwrap();
                match avail_a7 {
                    AvailableValue::Constant(y) => x.inst == BasicType::Ecall && y == &10,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
