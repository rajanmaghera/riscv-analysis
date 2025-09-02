use std::collections::HashSet;

use crate::analysis::LocValueMap;
use crate::parser::{HasIdentity, RVInstructionNode};
use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};

use super::{Cfg, CfgNode, RegisterSet};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct NodeWrapper {
    pub node: RVInstructionNode,
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
    pub func_exits: Vec<usize>,
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
    #[serde(default)]
    pub val_in: LocValueMap,
    #[serde(default)]
    pub val_out: LocValueMap,
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
                .labels()
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            func_entry: node
                .function()
                .iter()
                .map(|func| {
                    cfg.iter_source()
                        .position(|other| func.entry().id() == other.id())
                        .unwrap()
                })
                .collect::<Vec<_>>(),
            func_exits: node
                .function()
                .iter()
                .flat_map(|func| {
                    func.exits()
                        .iter()
                        .map(|exit| {
                            cfg.iter_source()
                                .position(|other| exit.id() == other.id())
                                .unwrap()
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            nexts: cfg
                .get_nexts(node)
                .map(|x| cfg.iter_source().position(|y| x.id() == y.id()).unwrap())
                .collect(),
            prevs: cfg
                .get_prevs(node)
                .map(|x| cfg.iter_source().position(|y| x.id() == y.id()).unwrap())
                .collect(),
            val_in: node.real_val_in(),
            val_out: node.real_val_out(),
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
        CfgWrapper(
            cfg.iter_source()
                .map(|x| NodeWrapper::from(x, cfg))
                .collect(),
        )
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
