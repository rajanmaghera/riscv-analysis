use super::{HasRawText, With};

impl<T> HasRawText for With<T> {
    fn raw_text(&self) -> &str {
        self.text.as_str()
    }
}
