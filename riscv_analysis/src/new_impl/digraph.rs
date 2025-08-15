use crate::new_impl::risc_v_implementation::RealInstList;
use crate::parser::HasIdentity;
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use uuid::Uuid;
// Implementer's notes: Digraph is split into two public structs, one for nodes and
// edges. This allows us to borrow from each independently, which leads to less
// headaches when modifying the graph.

pub struct DigraphEdges {
    nexts: HashMap<Uuid, HashSet<Uuid>>,
    prevs: HashMap<Uuid, HashSet<Uuid>>,
}
impl DigraphEdges {
    pub fn add_edge<T: HasIdentity>(&mut self, from: &T, to: &T) {
        if let Some(nexts_set) = self.nexts.get_mut(&from.id()) {
            nexts_set.insert(to.id());
        } else {
            self.nexts.insert(from.id(), HashSet::new());
        }

        if let Some(prevs_set) = self.prevs.get_mut(&to.id()) {
            prevs_set.insert(from.id());
        } else {
            self.prevs.insert(to.id(), HashSet::new());
        }
    }

    pub fn add_edges<'a, T: HasIdentity + 'a>(
        &'a mut self,
        edges: impl IntoIterator<Item = (&'a T, &'a T)>,
    ) {
        for (from, to) in edges {
            self.add_edge(from, to);
        }
    }

    pub fn remove_edge<T: HasIdentity>(&mut self, from: &T, to: &T) {
        if let Some(nexts_set) = self.nexts.get_mut(&from.id()) {
            nexts_set.remove(&to.id());
        }
        if let Some(prevs_set) = self.prevs.get_mut(&to.id()) {
            prevs_set.remove(&from.id());
        }
    }
}

pub struct DigraphNodes<T: HasIdentity> {
    items: HashMap<Uuid, T>,
}

impl<T: HasIdentity> DigraphNodes<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.values()
    }

    pub fn contains(&self, item: &T) -> bool {
        self.items.contains_key(&item.id())
    }
}

/// A directed graph structure.
pub struct Digraph<T: HasIdentity> {
    pub nodes: DigraphNodes<T>,
    pub edges: DigraphEdges,
}

impl<T: HasIdentity> Digraph<T> {
    pub fn new() -> Self {
        Self {
            nodes: DigraphNodes {
                items: HashMap::new(),
            },
            edges: DigraphEdges {
                nexts: HashMap::new(),
                prevs: HashMap::new(),
            },
        }
    }

    /// Map elements to different elements.
    ///
    /// This can be used to create an extra field
    /// or change the field when needed.
    pub fn map_all_elements<U: HasIdentity>(self, mut map: impl FnMut(T) -> U) -> Digraph<U> {
        Digraph {
            nodes: DigraphNodes {
                items: self
                    .nodes
                    .items
                    .into_iter()
                    .map(|(x, y)| (x, map(y)))
                    .collect(),
            },
            edges: self.edges,
        }
    }

    pub fn add_node(&mut self, item: T) {
        let id = item.id();
        self.nodes.items.insert(id, item);
        self.edges.prevs.insert(id, HashSet::new());
        self.edges.nexts.insert(id, HashSet::new());
    }

    pub fn remove_node(&mut self, item: &T) {
        if let Some(nexts_set) = self.edges.nexts.get_mut(&item.id()) {
            // Find all previous nodes and remove this entry
            for next in nexts_set.iter() {
                if let Some(prevs_set) = self.edges.prevs.get_mut(next) {
                    prevs_set.remove(&item.id());
                }
            }
        }
        if let Some(prevs_set) = self.edges.prevs.get_mut(&item.id()) {
            for prev in prevs_set.iter() {
                if let Some(nexts_set) = self.edges.nexts.get_mut(prev) {
                    nexts_set.remove(&item.id());
                }
            }
        }
        self.nodes.items.remove(&item.id());
    }

    pub fn get(&self, item: &T) -> &T {
        assert!(self.nodes.items.contains_key(&item.id()));
        self.nodes.items.get(&item.id()).unwrap()
    }

    pub fn get_many<'a>(&self, items: impl IntoIterator<Item = &'a T> + Clone) -> Vec<&T> where T: 'a {
        items.into_iter().map(|item| self.get(item)).collect()
    }

    pub fn get_by_id(&self, id: &Uuid) -> &T {
        assert!(self.nodes.items.contains_key(&id));
        self.nodes.items.get(id).unwrap()
    }

    pub fn get_many_by_ids<'a>(&self, ids: impl IntoIterator<Item = &'a Uuid> + Clone) -> Vec<&T> {
        ids.into_iter().map(|id| self.get_by_id(&id)).collect()
    }

    pub fn get_nexts(&self, item: &T) -> impl Iterator<Item = &T> + Clone {
        self.edges
            .nexts
            .get(&item.id())
            .unwrap()
            .iter()
            .map(|x| self.nodes.items.get(x).unwrap())
    }

    pub fn get_prevs(&self, item: &T) -> impl Iterator<Item = &T> + Clone {
        self.edges
            .prevs
            .get(&item.id())
            .unwrap()
            .iter()
            .map(|x| self.nodes.items.get(x).unwrap())
    }

    pub fn get_nexts_mut(&mut self, item: &T) -> impl Iterator<Item = &mut T> {
        let ids = self.edges.nexts.get(&item.id()).unwrap();
        self.nodes
            .items
            .iter_mut()
            .filter(move |x| !ids.contains(x.0))
            .map(|(_, x)| x)
    }

    pub fn get_prevs_mut(&mut self, item: &T) -> impl Iterator<Item = &mut T> {
        let ids = self.edges.prevs.get(&item.id()).unwrap();
        self.nodes
            .items
            .iter_mut()
            .filter(move |x| !ids.contains(x.0))
            .map(|(_, x)| x)
    }

    pub fn contains(&self, item: &T) -> bool {
        self.nodes.contains(item)
    }
}

impl<T: HasIdentity> FromIterator<T> for Digraph<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            nodes: DigraphNodes {
                items: iter.into_iter().map(|x| (x.id(), x)).collect(),
            },
            edges: DigraphEdges {
                nexts: HashMap::new(),
                prevs: HashMap::new(),
            },
        }
    }
}
/// A struct to create an annotated version of the original graph
///
/// This is not the best to use as it create a long-running reference
/// to the original graph. We cannot modify the original graph.
///
/// Prefer to map the map_all_elements item.
pub struct AnnotatedDigraph<'a, T: HasIdentity, U> {
    graph: &'a DigraphEdges,
    items: HashMap<Uuid, U>,
    _original_map: PhantomData<T>,
}

impl<'a, T: HasIdentity, U: Clone> AnnotatedDigraph<'a, T, U> {
    pub fn new(graph: &'a Digraph<T>, default_value: U) -> Self {
        Self {
            graph: &graph.edges,
            items: graph
                .nodes
                .items
                .iter()
                .map(|(id, _)| (*id, default_value.clone()))
                .collect(),
            _original_map: PhantomData,
        }
    }

    pub fn get(&self, item: &T) -> &U {
        assert!(self.items.contains_key(&item.id()));
        self.items.get(&item.id()).unwrap()
    }

    pub fn get_mut(&mut self, item: &T) -> &mut U {
        assert!(self.items.contains_key(&item.id()));
        self.items.get_mut(&item.id()).unwrap()
    }

    pub fn get_nexts(&self, item: &T) -> impl Iterator<Item = &U> {
        self.graph
            .nexts
            .get(&item.id())
            .unwrap()
            .iter()
            .map(|x| self.items.get(x).unwrap())
    }

    pub fn get_prevs(&self, item: &T) -> impl Iterator<Item = &U> {
        self.graph
            .prevs
            .get(&item.id())
            .unwrap()
            .iter()
            .map(|x| self.items.get(x).unwrap())
    }

    pub fn get_nexts_mut(&mut self, item: &T) -> impl Iterator<Item = &mut U> {
        let ids = self.graph.nexts.get(&item.id()).unwrap();
        self.items
            .iter_mut()
            .filter(move |(id, _)| !ids.contains(id))
            .map(|(_, x)| x)
    }

    pub fn get_prevs_mut(&mut self, item: &T) -> impl Iterator<Item = &mut U> {
        let ids = self.graph.prevs.get(&item.id()).unwrap();
        self.items
            .iter_mut()
            .filter(move |x| !ids.contains(x.0))
            .map(|(_, x)| x)
    }
}
