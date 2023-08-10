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
    /// Import and read a file into the reader
    ///
    /// Returns the UUID of the file and a peekable lexer. This lexer will allow
    /// you to search the file. Each file has its own attached lexer.
    fn import_file(
        &mut self,
        path: &str,
        in_file: Option<uuid::Uuid>,
    ) -> Result<(Uuid, Peekable<Lexer>), FileReaderError>;

    fn get_text(&self, uuid: uuid::Uuid) -> Option<String>;

    fn get_filename(&self, uuid: uuid::Uuid) -> Option<String>;
}
