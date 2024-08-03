#[derive(Debug)]
pub enum FileReaderError {
    IOErr(String),
    InternalFileNotFound,
    FileAlreadyRead(String),
    Unexpected,
    InvalidPath,
}
