use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    vec,
};

use crate::{
    cfg::regset::RegSets,
    parser::{ast::ASTNode, register::Register},
};

use super::{
    BasicBlock, BlockSet, DirectionMap, LabelToNode, LabelToNodes, NodeToNodes,
    NodeToPotentialLabel, CFG,
};

#[derive(Clone)]
pub struct Direction {
    pub next: BlockSet,
    pub prev: BlockSet,
}

pub struct DirectionalWrapper {
    pub cfg: CFG,
    pub directions: DirectionMap,
    pub node_function_map: NodeToPotentialLabel,
    // pub return_label_map: NodeToLabel,
    pub label_entry_map: LabelToNode,
    pub label_return_map: LabelToNodes,
    pub label_call_map: LabelToNodes,
    pub next_ast_map: NodeToNodes,
    pub prev_ast_map: NodeToNodes,
}

// TODO deprecate most of these
// TODO remove all "blocks" and treat each AST as their own block
// TODO if a node has previous nodes and it is not a func start or program start, remove its nexts. That way, we can see if code is unreachable based on if nexts/prevs are empty
impl ASTNode {
    // TODO BIG FIX for all different types of conditional/unconditional jumps
    // These defs are used to help start some functional analysis
    pub fn kill_available_value(&self) -> HashSet<Register> {
        match self.clone() {
            ASTNode::FuncEntry(_) => RegSets::caller_saved(),
            ASTNode::JumpLink(x) => {
                let mut set = RegSets::caller_saved();
                set.insert(x.rd.data);
                set
            }
            _ => self.kill(),
        }
    }

    pub fn kill(&self) -> HashSet<Register> {
        let regs: HashSet<Register> = match self.clone() {
            ASTNode::FuncEntry(_) => RegSets::callee_saved(),
            ASTNode::JumpLink(_) if self.is_function_call() => HashSet::new(),
            _ => self
                .stores_to()
                .map(|x| vec![x.data].into_iter().collect())
                .unwrap_or_default(),
        };
        regs.into_iter()
            .filter(|x| *x != Register::X0)
            .collect::<HashSet<_>>()
    }

    pub fn gen(&self) -> HashSet<Register> {
        let regs: HashSet<Register> = match self {
            ASTNode::JumpLinkR(_) if self.is_return() => RegSets::callee_saved(),
            _ => self.reads_from().into_iter().map(|x| x.data).collect(),
        };
        regs.into_iter()
            .filter(|x| *x != Register::X0)
            .collect::<HashSet<_>>()
    }
}
// calculate the in and out registers for every statement

impl CFG {
    pub fn calc_ast_directions(
        &self,
        direction_map: &HashMap<Rc<BasicBlock>, Direction>,
    ) -> (
        HashMap<Rc<ASTNode>, HashSet<Rc<ASTNode>>>,
        HashMap<Rc<ASTNode>, HashSet<Rc<ASTNode>>>,
    ) {
        let mut nexts = HashMap::new();
        let mut prevs = HashMap::new();
        for block in &self.blocks {
            let len = block.0.len();
            for (i, node) in block.0.iter().enumerate() {
                // determine next of each node
                let mut set = HashSet::new();
                if i == len - 1 {
                    let block = direction_map.get(block).unwrap().next.clone();
                    for next in block {
                        set.insert(next.0.first().unwrap().clone());
                    }
                } else {
                    set.insert(block.0[i + 1].clone());
                }
                nexts.insert(node.clone(), set);

                // determine prevs of each node
                let mut set = HashSet::new();
                if i == 0 {
                    let block = direction_map.get(block).unwrap().prev.clone();
                    for prev in block {
                        set.insert(prev.0.last().unwrap().clone());
                    }
                } else {
                    set.insert(block.0[i - 1].clone());
                }
                prevs.insert(node.clone(), set);
            }
        }
        (nexts, prevs)
    }
}

impl From<CFG> for DirectionalWrapper {
    fn from(cfg: CFG) -> Self {
        // initialize the direction map
        let mut direction_map = DirectionMap::new();
        for block in cfg.blocks.clone() {
            direction_map.insert(
                block.clone(),
                Direction {
                    next: HashSet::new(),
                    prev: HashSet::new(),
                },
            );
        }

        let mut prev: Option<Rc<BasicBlock>> = None;
        for block in cfg.blocks.clone() {
            for node in block.0.clone() {
                if let Some(n) = node.potential_jumps_to() {
                    // assert that this is the final node in the block
                    // assert_eq!(block.0.last().unwrap(), &node);
                    direction_map
                        .get_mut(&block)
                        .unwrap()
                        .next
                        .insert(cfg.labels.get(&n.data.0).unwrap().clone());
                    direction_map
                        .get_mut(cfg.labels.get(&n.data.0).unwrap())
                        .unwrap()
                        .prev
                        .insert(block.clone());
                }
            }

            // if the current block ends with a halt, we don't want to add it to the previous

            // LIN-SCAN
            if let Some(prev) = prev {
                direction_map
                    .get_mut(&prev)
                    .unwrap()
                    .next
                    .insert(block.clone());
                direction_map
                    .get_mut(&block)
                    .unwrap()
                    .prev
                    .insert(prev.clone());
            }

            // done weird because it's unstable in Rust
            prev = if let Some(fin) = block.0.last() {
                if fin.is_return() {
                    None
                } else {
                    Some(block.clone())
                }
            } else {
                Some(block.clone())
            }
        }

        // --- POST-DIRECTIONAL CALCULATIONS ---

        // AST NEXTS/PREVS
        // Using the big block nexts and prevs, we can now calculate the
        // nexts and prevs for each AST node. We do this by walking through
        let (next_ast_map, prev_ast_map) = cfg.calc_ast_directions(&direction_map);

        // TODO verify!!!
        // RETURN LABEL TARGETS
        // All return labels should only have one possible function entry
        // for good code, so we can just walk backwards from all return
        // labels till we reach an AST function start node. If we reach
        // multiple, we have a problem.
        let mut return_label_map = HashMap::new();
        let mut label_entry_map = HashMap::new();
        let mut return_block_map = HashMap::new();
        let mut label_return_map = HashMap::new();
        let mut node_function_map = HashMap::new();
        // for each return label
        for block in cfg.blocks.clone() {
            for node in &block.0.clone() {
                if node.is_return() {
                    // walk backwards
                    let mut walked = HashSet::new();
                    let mut queue = vec![node.clone()];
                    let mut found = Vec::new();
                    'inn: while let Some(n) = queue.pop() {
                        walked.insert(n.clone());
                        // if we find a function start, we're done
                        match n.as_ref() {
                            ASTNode::FuncEntry(x) => {
                                label_entry_map.insert(x.name.data.clone(), n.clone());
                                return_label_map.insert(node.clone(), x.name.clone());
                                match label_return_map.get_mut(&x.name.data) {
                                    None => {
                                        let mut new_set = HashSet::new();
                                        new_set.insert(node.clone());
                                        label_return_map.insert(x.name.data.clone(), new_set);
                                    }
                                    Some(x) => {
                                        x.insert(node.clone());
                                    }
                                }
                                match return_block_map.get_mut(&x.name.data) {
                                    None => {
                                        let mut new_set = HashSet::new();
                                        new_set.insert(block.clone());
                                        return_block_map.insert(x.name.data.clone(), new_set);
                                    }
                                    Some(x) => {
                                        x.insert(block.clone());
                                    }
                                }
                                found.push(x.name.clone());

                                continue 'inn;
                            }
                            _ => (),
                        }
                        // otherwise, add all prevs to the queue
                        for prev in prev_ast_map.get(&n).unwrap() {
                            if !walked.contains(prev) {
                                queue.push(prev.clone());
                            }
                        }
                    }
                    // if we found anything other than one, it is not SESE
                    if found.len() > 1 {
                        unimplemented!("Multiple function starts found for return label");
                    } else if found.is_empty() {
                        unimplemented!("No function starts found for return label");
                    }

                    // if we found one, add all the walked nodes to the node_function_map
                    for node in walked {
                        node_function_map.insert(node, found[0].clone());
                    }
                }
            }
        }

        // LABEL CALL MAP
        // Find all places where a label is called and add them to the
        // label call map
        let mut label_call_map = HashMap::new();
        for block in cfg.blocks.clone() {
            for node in &block.0.clone() {
                if let ASTNode::JumpLink(x) = node.as_ref() {
                    match label_call_map.get_mut(&x.name.data) {
                        None => {
                            let mut new_set = HashSet::new();
                            new_set.insert(node.clone());
                            label_call_map.insert(x.name.data.clone(), new_set);
                        }
                        Some(x) => {
                            x.insert(node.clone());
                        }
                    }
                }
            }
        }

        // FUNCTION CALL GRAPH
        // TODO maybe??

        // JUMP TARGETS
        // TODO find all targets of branches and add them to the next set
        // If we have made our CFG correctly, all possible branches should only
        // ever be at the end of a block, so we can just look at the last node
        // of each block

        // calculate the possible function labels

        Self {
            cfg,
            next_ast_map,
            prev_ast_map,
            node_function_map,
            label_entry_map,
            label_return_map,
            label_call_map,
            directions: direction_map,
        }
    }
}
