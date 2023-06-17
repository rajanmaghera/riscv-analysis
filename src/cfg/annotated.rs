use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::{format, Display},
    hash::{Hash, Hasher},
    rc::Rc,
};

use itertools::Itertools;

use crate::parser::{
    ast::{ASTNode, Label, LabelString},
    inst::BasicType,
    register::Register,
    token::WithToken,
};

use super::{
    regset::RegSets, AvailableValue, AvailableValueResult, BasicBlock, DirectionMap,
    DirectionalWrapper, LabelToNode, LabelToNodes, LiveAnalysisResult, NodeToNodes,
    NodeToPotentialLabel, CFG,
};

pub struct AnnotatedCFG {
    // TODO convert all maps from nodes/indices to fields on the node, so there's
    // no get nonsense
    // TODO convert each map that is from a function to a struct so that all of them
    // share the same struct from one call, same with direction
    pub blocks: Vec<Rc<BasicBlock>>,
    pub nodes: Vec<Rc<ASTNode>>,
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
        for block in self.blocks.iter() {
            f.write_str("+---------\n")?;
            f.write_str(&format!(
                "| LABELS: {:?}, ID: {}\n",
                labels.next().unwrap(),
                block.1.as_simple().to_string()[..8].to_string()
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
            for node in block.0.iter() {
                f.write_str(&format!(
                    "| {:>3}: {}\n|  in: {:<20}\n| out: {:<20}\n",
                    index,
                    node,
                    self.liveness
                        .live_in
                        .get(index)
                        .unwrap()
                        .into_iter()
                        .sorted()
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.liveness
                        .live_out
                        .get(index)
                        .unwrap()
                        .into_iter()
                        .sorted()
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                ))?;
                f.write_str(&format!(
                    "| val: {}\n",
                    self.available
                        .avail_out
                        .get(index)
                        .unwrap()
                        .into_iter()
                        .sorted_by_key(|x| x.0)
                        .into_iter()
                        .map(|(k, v)| format!("[{}: {}]", k, v))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))?;
                f.write_str(&format!(
                    "| stk: {}\n",
                    self.available
                        .stack_out
                        .get(index)
                        .unwrap()
                        .into_iter()
                        .sorted_by_key(|x| x.0)
                        .into_iter()
                        .map(|(k, v)| format!("[{}: {}]", k, v))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))?;
                f.write_str(&format!(
                    "| udf: {}\n",
                    self.liveness
                        .uncond_defs
                        .get(index)
                        .unwrap()
                        .into_iter()
                        .sorted()
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))?;
                index += 1;
            }
            f.write_str("+---------\n")?;
        }
        f.write_str("FUNCTION DATA:\n")?;
        for (k, _) in self.label_entry_map.iter() {
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
            .get(&crate::parser::ast::LabelString(name.to_owned()))?;
        let idx = self.nodes.iter().position(|x| x == val)?;
        let node = self.liveness.live_in.get(idx)?.clone();
        let node = node.intersection(&RegSets::argument()).cloned().collect();
        Some(node)
    }

    pub fn function_rets(&self, name: &str) -> Option<HashSet<Register>> {
        let val = self
            .label_return_map
            .get(&crate::parser::ast::LabelString(name.to_owned()))?
            .into_iter()
            .next()?;
        let idx = self.nodes.iter().position(|x| x == val)?;
        let node = self.liveness.live_in.get(idx)?.clone();
        let node = node.intersection(&RegSets::ret()).cloned().collect();
        Some(node)
    }

    pub fn is_program_exit(&self, node: &Rc<ASTNode>) -> bool {
        match &*(*node) {
            ASTNode::Basic(x) => {
                let idx = self.nodes.iter().position(|x| x == node).unwrap();

                let avail_a7 = self
                    .available
                    .avail_out
                    .get(idx)
                    .unwrap()
                    .get(&Register::X17)
                    .unwrap();
                match avail_a7 {
                    AvailableValue::Constant(y) => x.inst.data == BasicType::Ecall && y == &10,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
