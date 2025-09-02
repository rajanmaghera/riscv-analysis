use crate::parser::HasIdentity;
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use uuid::Uuid;

/// A worklist iterator that allows pushing elements to it while
/// looping, and only iterating elements that have not been visited.
///
/// The worklist uses RefCells, but it is implemented in
/// a way that is safe to do so, thus could be removed and
/// replaced with unsafe code.
pub struct WorklistVisitedIter<T>
where
    T: HasIdentity,
{
    worklist: RefCell<VecDeque<T>>,
    visited: RefCell<HashSet<Uuid>>,
}

impl<T> WorklistVisitedIter<T>
where
    T: HasIdentity,
{
    pub fn new(initial: impl IntoIterator<Item = T>) -> Self {
        Self {
            worklist: RefCell::new(VecDeque::from_iter(initial)),
            visited: RefCell::new(HashSet::new()),
        }
    }

    /// Add new items to the worklist. Can be called during iteration.
    pub fn add(&self, item: T) {
        if !self.visited.borrow().contains(&item.id()) {
            self.visited.borrow_mut().insert(item.id());
            self.worklist.borrow_mut().push_back(item);
        }
    }
}

impl<T> Iterator for &WorklistVisitedIter<T>
where
    T: HasIdentity,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.worklist.borrow_mut().pop_front()
    }
}
