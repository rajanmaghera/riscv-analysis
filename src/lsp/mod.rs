// Type conversions for LSP

use std::collections::HashMap;
use std::convert::From;
use std::iter::Peekable;

use crate::parser::{Lexer, RVParser, Range as MyRange};
use crate::passes::{DiagnosticItem, DiagnosticLocation, DiagnosticMessage};
use crate::passes::{WarningLevel};
use crate::reader::{FileReader, FileReaderError};
use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, Position, Range,
};

mod completion;
pub use completion::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use url::Url;
use uuid::Uuid;
use wasm_bindgen::JsValue;

impl From<&MyRange> for Range {
    fn from(r: &MyRange) -> Self {
        lsp_types::Range {
            start: Position {
                line: r.start.line.try_into().unwrap_or(0),
                character: r.start.column.try_into().unwrap_or(0),
            },
            end: Position {
                line: r.end.line.try_into().unwrap_or(0),
                character: r.end.column.try_into().unwrap_or(0),
            },
        }
    }
}

impl From<WarningLevel> for DiagnosticSeverity {
    fn from(w: WarningLevel) -> Self {
        match w {
            WarningLevel::Warning => DiagnosticSeverity::WARNING,
            WarningLevel::Error => DiagnosticSeverity::ERROR,
        }
    }
}

impl DiagnosticItem {
    pub fn to_lsp_diag(&self, parser: &RVParser<LSPFileReader>) -> LSPRVSingleDiagnostic {
        LSPRVSingleDiagnostic {
            uri: parser
                .reader
                .get_filename(self.file)
                .unwrap_or(String::new()),
            diagnostic: Diagnostic {
                range: (&self.range).into(),
                severity: Some(self.level.clone().into()),
                code: None,
                code_description: None,
                source: None,
                message: self.long_description.clone(),
                related_information: self.related.clone().map(|f| {
                    f.into_iter()
                        .map(|f| DiagnosticRelatedInformation {
                            location: Location {
                                uri: Url::parse(
                                    &parser.reader.get_filename(f.file).unwrap_or(String::new()),
                                )
                                .unwrap(),
                                range: (&f.range).into(),
                            },
                            message: f.description,
                        })
                        .collect::<Vec<_>>()
                }),
                tags: None,
                data: None,
            },
        }
    }
}

pub trait CanGetURIString: FileReader {
    fn get_uri_string(&self, uuid: Uuid) -> LSPRVDocument;
}

#[derive(Clone)]
pub struct LSPFileReader {
    pub file_uris: HashMap<Uuid, LSPRVDocument>,
}

#[derive(Deserialize, Clone)]
pub struct LSPRVDocument {
    pub uri: String,
    pub text: String,
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

// WASM MODULES

#[derive(Default)]
pub struct WrapperDiag(pub Vec<Diagnostic>);

impl From<WrapperDiag> for JsValue {
    fn from(w: WrapperDiag) -> Self {
        to_value(&w.0).unwrap()
    }
}

impl WrapperDiag {
    fn new(str: &str) -> Self {
        let diag = Diagnostic::new_simple(
            Range::new(
                Position {
                    line: 1,
                    character: 1,
                },
                Position {
                    line: 1,
                    character: 2,
                },
            ),
            str.to_owned(),
        );
        WrapperDiag(vec![diag])
    }
}

impl CanGetURIString for LSPFileReader {
    fn get_uri_string(&self, uuid: Uuid) -> LSPRVDocument {
        self.file_uris.get(&uuid).unwrap().clone()
    }
}

impl FileReader for LSPFileReader {
    fn get_filename(&self, uuid: uuid::Uuid) -> Option<String> {
        self.file_uris.get(&uuid).map(|x| x.uri.clone())
    }

    fn import_file(
        &mut self,
        path: &str,
        in_file: Option<uuid::Uuid>,
    ) -> Result<(Uuid, Peekable<Lexer>), FileReaderError> {
        // if there is an in_file, find its path and use that as the parent
        let fulluri = match in_file {
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
        let lexer = Lexer::new(&doc.1.text, doc.0);
        Ok((doc.0, lexer.peekable()))
    }
}

impl LSPFileReader {
    pub fn new(docs: Vec<LSPRVDocument>) -> Self {
        let mut map = HashMap::new();

        for doc in docs {
            let uuid = Uuid::new_v4();
            map.insert(uuid, doc);
        }

        LSPFileReader { file_uris: map }
    }
}
