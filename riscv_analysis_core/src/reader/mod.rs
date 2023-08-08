use std::iter::Peekable;

use uuid::Uuid;

use crate::parser::Lexer;

#[derive(Debug)]
pub enum FileReaderError {
    IOErr(String),
    InternalFileNotFound,
    FileAlreadyRead(String),
    Unexpected,
    InvalidPath,
}

pub trait FileReader: Sized {
    fn import_file(
        &mut self,
        path: &str,
        in_file: Option<uuid::Uuid>,
    ) -> Result<(Uuid, Peekable<Lexer>), FileReaderError>;

    fn get_filename(&self, uuid: uuid::Uuid) -> Option<String>;
}
