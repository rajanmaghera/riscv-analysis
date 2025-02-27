use crate::parser::HasRawText;

pub trait HasRawTextOwned {
    fn raw_text_owned(&self) -> String;
}

impl<T: HasRawText> HasRawTextOwned for T {
    fn raw_text_owned(&self) -> String {
        self.raw_text().to_owned()
    }
}
