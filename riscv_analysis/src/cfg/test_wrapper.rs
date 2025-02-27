use std::collections::HashSet;

use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    analysis::MemoryLocation,
    parser::{HasIdentity, ParserNode, Register},
};

use super::{AvailableValueMap, Cfg, CfgNode, RegisterSet};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct NodeWrapper {
    pub node: ParserNode,
    // skip if empty
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "sorted_set"
    )]
    pub labels: HashSet<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub func_entry: Vec<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub func_exit: Vec<usize>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "sorted_set"
    )]
    pub nexts: HashSet<usize>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "sorted_set"
    )]
    pub prevs: HashSet<usize>,
    #[serde(default, skip_serializing_if = "AvailableValueMap::is_empty")]
    pub reg_values_in: AvailableValueMap<Register>,
    #[serde(default, skip_serializing_if = "AvailableValueMap::is_empty")]
    pub reg_values_out: AvailableValueMap<Register>,
    #[serde(default, skip_serializing_if = "AvailableValueMap::is_empty")]
    pub memory_values_in: AvailableValueMap<MemoryLocation>,
    #[serde(default, skip_serializing_if = "AvailableValueMap::is_empty")]
    pub memory_values_out: AvailableValueMap<MemoryLocation>,
    #[serde(default, skip_serializing_if = "RegisterSet::is_empty")]
    pub live_in: RegisterSet,
    #[serde(default, skip_serializing_if = "RegisterSet::is_empty")]
    pub live_out: RegisterSet,
    #[serde(default, skip_serializing_if = "RegisterSet::is_empty")]
    pub u_def: RegisterSet,
}

impl NodeWrapper {
    fn from(node: &CfgNode, cfg: &Cfg) -> Self {
        NodeWrapper {
            node: node.node(),
            labels: node
                .labels
                .iter()
                .map(|x| x.get().0.clone())
                .collect(),
            func_entry: node
                .functions()
                .iter()
                .map(|func| {
                    cfg.iter()
                        .position(|other| func.entry().id() == other.id())
                        .unwrap()
                })
                .collect::<Vec<_>>(),
            func_exit: node
                .functions()
                .iter()
                .map(|func| {
                    cfg.iter()
                        .position(|other| func.exit().id() == other.id())
                        .unwrap()
                })
                .collect::<Vec<_>>(),
            nexts: node
                .nexts()
                .iter()
                .map(|x| cfg.iter().position(|y| x.id() == y.id()).unwrap())
                .collect(),
            prevs: node
                .prevs()
                .iter()
                .map(|x| cfg.iter().position(|y| x.id() == y.id()).unwrap())
                .collect(),
            reg_values_in: node.reg_values_in(),
            reg_values_out: node.reg_values_out(),
            memory_values_in: node.memory_values_in(),
            memory_values_out: node.memory_values_out(),
            live_in: node.live_in(),
            live_out: node.live_out(),
            u_def: node.u_def(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct CfgWrapper(Vec<NodeWrapper>);

impl From<&Cfg> for CfgWrapper {
    fn from(cfg: &Cfg) -> Self {
        CfgWrapper(cfg.iter().map(|x| NodeWrapper::from(&x, cfg)).collect())
    }
}

pub fn sorted_set<S: Serializer, V: Serialize + Ord, H: std::hash::BuildHasher>(
    value: &HashSet<V, H>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    value
        .iter()
        .sorted()
        .collect::<Vec<_>>()
        .serialize(serializer)
}
