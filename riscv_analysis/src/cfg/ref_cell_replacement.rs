use std::cell::RefCell;

pub trait RefCellReplacement<T: PartialEq> {
    fn replace_if_changed(&self, new: T) -> bool;
}

impl<T: PartialEq> RefCellReplacement<T> for RefCell<T> {
    fn replace_if_changed(&self, new: T) -> bool {
        let mut original = self.borrow_mut();
        if *original == new {
            false
        } else {
            *original = new;
            true
        }
    }
}
