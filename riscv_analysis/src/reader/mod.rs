use uuid::Uuid;

#[derive(Debug)]
pub enum FileReaderError {
    IOErr(String),
    InternalFileNotFound,
    FileAlreadyRead(String),
    Unexpected,
    InvalidPath,
}

pub trait FileReader: Sized {
    /// Import and read a file into the reader.
    ///
    /// Returns the UUID of the file and a string containing the file's contents.
    fn import_file(
        &mut self,
        path: &str,
        parent_file: Option<uuid::Uuid>,
    ) -> Result<(Uuid, String), FileReaderError>;

    fn get_text(&self, uuid: uuid::Uuid) -> Option<String>;

    fn get_filename(&self, uuid: uuid::Uuid) -> Option<String>;
}
