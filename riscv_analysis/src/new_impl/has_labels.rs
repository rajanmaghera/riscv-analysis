use std::collections::HashSet;

pub trait HasLabels {
    fn get_labels(&self) -> &HashSet<String>;

    fn has_label(&self, label: &impl ToString) -> bool;
}
