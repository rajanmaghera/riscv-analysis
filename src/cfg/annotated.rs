use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::parser::ast::ASTNode;

use super::{
    AvailableValueResult, BasicBlock, DirectionMap, DirectionalWrapper, LabelToNode, LabelToNodes,
    LiveAnalysisResult, NodeToNodes, CFG,
};

pub struct AnnotatedCFG {
    pub blocks: Vec<Rc<BasicBlock>>,
    pub nodes: Vec<Rc<ASTNode>>,
    pub labels: HashMap<String, Rc<BasicBlock>>,
    pub labels_for_branch: Vec<Vec<String>>,
    pub directions: DirectionMap,
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
                    "| IN: {:<20} | OUT: {:<20} {}\n",
                    self.liveness
                        .live_in
                        .get(index)
                        .unwrap()
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.liveness
                        .live_out
                        .get(index)
                        .unwrap()
                        .into_iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    node
                ))?;
                f.write_str(&format!(
                    "| {}\n",
                    self.available
                        .avail_out
                        .get(index)
                        .unwrap()
                        .into_iter()
                        .map(|(k, v)| format!("[{}: {}]", k, v))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))?;
                index += 1;
            }
            f.write_str("+---------\n")?;
        }
        Ok(())
    }
}
