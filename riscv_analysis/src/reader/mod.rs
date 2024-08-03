use crate::parser::Lexer;
use uuid::Uuid;

mod error;
pub use error::*;

mod full_lexer;
pub use full_lexer::*;

mod lexing_string;
pub use lexing_string::*;

/// An item that can read and keep track of files that will be read.
pub trait FileReader: Sized + Clone {
    /// Import and read a file into the reader
    ///
    /// Returns the UUID of the file and a lexer. This lexer will allow
    /// you to search the file. Each file has its own attached lexer.
    fn import_file(
        &mut self,
        path: &str,
        in_file: Option<Uuid>,
    ) -> Result<LexingString, FileReaderError>;

    fn get_text(&self, uuid: uuid::Uuid) -> Option<&str>;

    fn get_filename(&self, uuid: uuid::Uuid) -> Option<&str>;
}
