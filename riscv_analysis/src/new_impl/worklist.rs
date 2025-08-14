use std::cell::RefCell;
use std::collections::VecDeque;

/// A worklist iterator that allows pushing elements to it.
///
/// The worklist uses RefCells, but it is implemented in
/// a way that is safe to do so, thus could be removed and
/// replaced with unsafe code.
pub struct WorklistIter<T> {
    worklist: RefCell<VecDeque<T>>,
}

impl<T> WorklistIter<T> {
    pub fn new(initial: impl IntoIterator<Item = T>) -> Self {
        Self {
            worklist: RefCell::new(VecDeque::from_iter(initial)),
        }
    }

    /// Add new items to the worklist. Can be called during iteration.
    pub fn add(&self, item: T) {
        // TODO: only add elements if not in program
        self.worklist.borrow_mut().push_back(item);
    }

    /// Add iterable items to worklist.
    pub fn add_many(&self, items: impl IntoIterator<Item = T>) {
        self.worklist.borrow_mut().extend(items);
    }
}

impl<T> Iterator for &WorklistIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.worklist.borrow_mut().pop_front()
    }
}
