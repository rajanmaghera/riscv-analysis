use std::collections::{BTreeMap, HashMap, HashSet};

use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};

use crate::{
    analysis::AvailableValue,
    parser::{ParserNode, Register},
};

use super::{CFGNode, Cfg};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub func_entry: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub func_exit: Option<usize>,
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
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        serialize_with = "sorted_map"
    )]
    pub reg_values_in: HashMap<Register, AvailableValue>,
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        serialize_with = "sorted_map"
    )]
    pub reg_values_out: HashMap<Register, AvailableValue>,
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        serialize_with = "sorted_map"
    )]
    pub stack_values_in: HashMap<i32, AvailableValue>,
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        serialize_with = "sorted_map"
    )]
    pub stack_values_out: HashMap<i32, AvailableValue>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "sorted_set"
    )]
    pub live_in: HashSet<Register>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "sorted_set"
    )]
    pub live_out: HashSet<Register>,
    #[serde(
        default,
        skip_serializing_if = "HashSet::is_empty",
        serialize_with = "sorted_set"
    )]
    pub u_def: HashSet<Register>,
}

impl NodeWrapper {
    fn from(node: &CFGNode, cfg: &Cfg) -> Self {
        NodeWrapper {
            node: node.node(),
            labels: node.labels.iter().map(|x| x.data.0.clone()).collect(),
            func_entry: node.function().clone().map(|x| {
                cfg.nodes
                    .iter()
                    .position(|y| x.entry.node().id() == y.node().id())
                    .unwrap()
            }),
            func_exit: node.function().clone().map(|x| {
                cfg.nodes
                    .iter()
                    .position(|y| x.exit.node().id() == y.node().id())
                    .unwrap()
            }),
            nexts: node
                .nexts()
                .iter()
                .map(|x| {
                    cfg.nodes
                        .iter()
                        .position(|y| x.node().id() == y.node().id())
                        .unwrap()
                })
                .collect(),
            prevs: node
                .prevs()
                .iter()
                .map(|x| {
                    cfg.nodes
                        .iter()
                        .position(|y| x.node().id() == y.node().id())
                        .unwrap()
                })
                .collect(),
            reg_values_in: node.reg_values_in(),
            reg_values_out: node.reg_values_out(),
            stack_values_in: node.stack_values_in(),
            stack_values_out: node.stack_values_out(),
            live_in: node.live_in(),
            live_out: node.live_out(),
            u_def: node.u_def(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct CFGWrapper(Vec<NodeWrapper>);

impl From<&Cfg> for CFGWrapper {
    fn from(cfg: &Cfg) -> Self {
        CFGWrapper(
            cfg.nodes
                .iter()
                .map(|x| NodeWrapper::from(x, cfg))
                .collect(),
        )
    }
}

pub fn sorted_map<S: Serializer, K: Serialize + Ord, V: Serialize, H: std::hash::BuildHasher>(
    value: &HashMap<K, V, H>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    value
        .iter()
        .sorted_by_key(|v| v.0)
        .collect::<BTreeMap<_, _>>()
        .serialize(serializer)
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

pub fn sorted_vec<S: Serializer, V: Serialize + Ord>(
    value: &[V],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    value
        .iter()
        .sorted()
        .collect::<Vec<_>>()
        .serialize(serializer)
}
