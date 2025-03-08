use uuid::Uuid;

use crate::reader::{FileReader, FileReaderError};

/// A file reader that cannot read from any paths.
///
/// This file reader is useful when we want to create a
/// parser that needs a file reader, but we know that the
/// parser will not need to read from any files or cannot
/// read from any files. This is useful for testing.
///
/// Using this file reader will prevent the usage of
/// any `.include` directives in the parser. Every
/// use of `.include` will result in an error.
#[derive(Debug, Clone)]
pub struct EmptyFileReader {
    base_file_contents: String,
    base_file_uuid: Option<Uuid>,
}

impl EmptyFileReader {
    /// Create a new empty file reader.
    ///
    /// An `EmptyFileReader` is a file reader that cannot read from any paths.
    /// It takes in the file text that the base file should contain.
    #[must_use]
    pub fn new(text: &str) -> Self {
        Self {
            base_file_contents: text.to_string(),
            base_file_uuid: None,
        }
    }

    /// Get the "fake" file path used for the base file.
    ///
    /// The interface for a file reader requires a notion of a file path or file reader.
    /// This function returns the file path that the base file is expected to have.
    ///
    /// ```
    /// use riscv_analysis::parser::EmptyFileReader;
    /// assert_eq!(EmptyFileReader::get_file_path(), "base_file.s");
    /// ```
    #[must_use]
    pub fn get_file_path() -> &'static str {
        "base_file.s"
    }
}

impl FileReader for EmptyFileReader {
    fn import_file(
        &mut self,
        path: &str,
        parent_file: Option<uuid::Uuid>,
    ) -> Result<(Uuid, String), FileReaderError> {
        if parent_file.is_some() {
            Err(FileReaderError::Unexpected)
        } else if self.base_file_uuid.is_some() {
            Err(FileReaderError::FileAlreadyRead(
                Self::get_file_path().to_string(),
            ))
        } else if path != Self::get_file_path() {
            Err(FileReaderError::InternalFileNotFound)
        } else {
            let uuid = uuid::Uuid::new_v4();
            self.base_file_uuid = Some(uuid);
            Ok((uuid, self.base_file_contents.clone()))
        }
    }

    fn get_text(&self, uuid: uuid::Uuid) -> Option<String> {
        if uuid == self.base_file_uuid? {
            Some(self.base_file_contents.clone())
        } else {
            None
        }
    }

    fn get_filename(&self, uuid: uuid::Uuid) -> Option<String> {
        if uuid == self.base_file_uuid? {
            Some(Self::get_file_path().to_string())
        } else {
            None
        }
    }

    fn get_base_file(&self) -> Option<uuid::Uuid> {
        self.base_file_uuid
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn can_read_from_text() {
        let text = "Hello, world!";
        let mut reader = EmptyFileReader::new(text);

        let (uuid, contents) = reader
            .import_file(EmptyFileReader::get_file_path(), None)
            .expect("File reading should not fail");
        assert_eq!(contents, text);

        let text = reader
            .get_text(uuid)
            .expect("File path should exist in reader");
        assert_eq!(text, "Hello, world!");

        let filename = reader
            .get_filename(uuid)
            .expect("File name should exist for base reader");
        assert_eq!(filename, EmptyFileReader::get_file_path());
    }

    #[test]
    fn can_get_error_for_invalid_paths() {
        let text = "Hello, world!";
        let mut reader = EmptyFileReader::new(text);

        let result = reader.import_file("invalid_path.s", None);
        assert!(matches!(result, Err(FileReaderError::InternalFileNotFound)));
    }

    #[test]
    fn can_get_error_for_including_a_parent_uuid() {
        let text = "Hello, world!";
        let mut reader = EmptyFileReader::new(text);

        let result = reader.import_file(EmptyFileReader::get_file_path(), Some(Uuid::new_v4()));
        assert!(matches!(result, Err(FileReaderError::Unexpected)));

        let result = reader.import_file(EmptyFileReader::get_file_path(), Some(Uuid::nil()));
        assert!(matches!(result, Err(FileReaderError::Unexpected)));
    }

    #[test]
    fn can_read_empty_text() {
        let text = "";
        let mut reader = EmptyFileReader::new(text);

        let (uuid, contents) = reader
            .import_file(EmptyFileReader::get_file_path(), None)
            .expect("File reading should not fail");
        assert_eq!(contents, text);

        let text = reader
            .get_text(uuid)
            .expect("File path should exist in reader");
        assert_eq!(text, "");
    }
}
