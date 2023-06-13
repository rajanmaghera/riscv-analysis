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

