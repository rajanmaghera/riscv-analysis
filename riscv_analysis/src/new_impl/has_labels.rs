pub trait HasLabels {
    fn get_labels(&self) -> impl Iterator<Item = &String>;

    fn has_label(&self, label: &impl ToString) -> bool;
}
