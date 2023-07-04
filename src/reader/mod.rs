use std::iter::Peekable;

use uuid::Uuid;

use crate::parser::{Lexer, With};

#[derive(Debug)]
pub enum FileReaderError {
    IOError(std::io::Error),
    InternalFileNotFound,
    FileAlreadyRead(String),
}

pub trait FileReader: Sized {
    fn import_file(
        &mut self,
        path: &str,
        in_file: Option<uuid::Uuid>,
    ) -> Result<(Uuid, Peekable<Lexer>), FileReaderError>;

    fn get_filename(&self, uuid: uuid::Uuid) -> String;
}
