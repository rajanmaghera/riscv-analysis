// Type conversions for LSP

use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, Position, Range,
};
use riscv_analysis::parser::{CanGetURIString, RVDocument, RVParser, Range as MyRange};
use riscv_analysis::passes::DiagnosticItem;
use riscv_analysis::passes::SeverityLevel;
use riscv_analysis::reader::{FileReader, FileReaderError};
use std::collections::HashMap;

mod completion;
pub use completion::*;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

trait RangeInto {
    fn to_range(&self) -> Range;
}

impl RangeInto for MyRange {
    fn to_range(&self) -> Range {
        lsp_types::Range {
            start: Position {
                line: self.start().zero_idx_line().try_into().unwrap_or(0),
                character: self.start().zero_idx_column().try_into().unwrap_or(0),
            },
            end: Position {
                line: self.end().zero_idx_line().try_into().unwrap_or(0),
                character: self.end().zero_idx_column().try_into().unwrap_or(0),
            },
        }
    }
}

trait WarningInto {
    fn to_severity(&self) -> DiagnosticSeverity;
}

impl WarningInto for SeverityLevel {
    fn to_severity(&self) -> DiagnosticSeverity {
        match self {
            SeverityLevel::Error => DiagnosticSeverity::ERROR,
            SeverityLevel::Warning => DiagnosticSeverity::WARNING,
            SeverityLevel::Information => DiagnosticSeverity::INFORMATION,
            SeverityLevel::Hint => DiagnosticSeverity::HINT,
        }
    }
}

pub trait LSPDiag {
    fn to_lsp_diag(&self, parser: &RVParser<LSPFileReader>) -> LSPRVSingleDiagnostic;
}

impl LSPDiag for DiagnosticItem {
    fn to_lsp_diag(&self, parser: &RVParser<LSPFileReader>) -> LSPRVSingleDiagnostic {
        LSPRVSingleDiagnostic {
            uri: parser.reader.get_filename(self.file).unwrap_or_default(), // Empty string by default
            diagnostic: Diagnostic {
                range: self.range.to_range(),
                severity: Some(self.level.clone().to_severity()),
                code: None,
                code_description: None,
                source: None,
                message: self.long_description.clone(),
                related_information: self.related.clone().map(|f| {
                    f.into_iter()
                        .map(|f1| DiagnosticRelatedInformation {
                            location: Location {
                                uri: Url::parse(
                                    &parser.reader.get_filename(f1.file).unwrap_or_default(), // Empty string by default
                                )
                                .unwrap(),
                                range: f1.range.to_range(),
                            },
                            message: f1.description,
                        })
                        .collect::<Vec<_>>()
                }),
                tags: None,
                data: None,
            },
        }
    }
}

#[derive(Clone)]
pub struct LSPFileReader {
    pub file_uris: HashMap<Uuid, RVDocument>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LSPRVDiagnostic {
    pub uri: String,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct LSPRVSingleDiagnostic {
    pub uri: String,
    pub diagnostic: Diagnostic,
}

impl CanGetURIString for LSPFileReader {
    fn get_uri_string(&self, uuid: Uuid) -> RVDocument {
        self.file_uris.get(&uuid).unwrap().clone()
    }
}

impl FileReader for LSPFileReader {
    fn get_text(&self, uuid: uuid::Uuid) -> Option<String> {
        self.file_uris.get(&uuid).map(|x| x.text.clone())
    }
    fn get_filename(&self, uuid: uuid::Uuid) -> Option<String> {
        self.file_uris.get(&uuid).map(|x| x.uri.clone())
    }

    fn import_file(
        &mut self,
        path: &str,
        parent_file: Option<uuid::Uuid>,
    ) -> Result<(Uuid, String), FileReaderError> {
        // if there is an parent_file, find its path and use that as the parent
        let fulluri = match parent_file {
            Some(uuid) => {
                let doc = self.file_uris.get(&uuid).unwrap();
                let uri = lsp_types::Url::parse(&doc.uri).unwrap();
                let fileuri = uri.join(path).unwrap();
                fileuri.to_string()
            }
            // otherwise, this is the full path to the file, denoted by its uri
            None => lsp_types::Url::parse(path).unwrap().to_string(),
        };

        // find file in values of hashmap
        let doc = self
            .file_uris
            .clone()
            .into_iter()
            .find(|x| x.1.uri == fulluri);

        // if file not found, return error
        if doc.is_none() {
            return Err(FileReaderError::InternalFileNotFound);
        }

        // if file found, return lexer
        let doc = doc.unwrap();
        Ok((doc.0, doc.1.text))
    }
}

impl LSPFileReader {
    pub fn new(docs: Vec<RVDocument>) -> Self {
        let mut map = HashMap::new();

        for doc in docs {
            let uuid = Uuid::new_v4();
            map.insert(uuid, doc);
        }

        LSPFileReader { file_uris: map }
    }
}
