use crate::cfg::Cfg;
use crate::parser::{DirectiveType, ParserNode, RVParser};
use crate::passes::Manager;
use lsp_types::{Diagnostic, Position, Range};
use parser::Lexer;
use reader::{FileReader, FileReaderError};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use std::collections::HashSet;
use std::{collections::HashMap, iter::Peekable};
use uuid::Uuid;
use wasm_bindgen::prelude::*;
mod analysis;
mod cfg;
mod gen;
mod lints;
mod lsp;
mod parser;
mod passes;
mod reader;

// WASM MODULES

#[wasm_bindgen]
pub fn riscv_parse(input: &str) -> Result<String, String> {
    Ok(format!("Hello, {}!", input))
}

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

// impl From<Vec<>> for WrapperDiag {
//     fn from(e: PassErrors) -> Self {
//         let mut diag = Vec::new();
//         for err in e.errors {
//             diag.push(Diagnostic::from(err));
//         }
//         WrapperDiag(diag)
//     }
// }

struct LSPFileReader {
    file_uris: HashMap<Uuid, LSPRVDocument>,
}

impl<T> RVParser<T>
where
    T: CanGetURIString,
{
    fn get_full_url(&self, path: &str, uuid: Uuid) -> String {
        let doc = self.reader.get_uri_string(uuid);
        let uri = lsp_types::Url::parse(&doc.uri).unwrap();
        let fileuri = uri.join(path).unwrap();
        fileuri.to_string()
    }
}

trait CanGetURIString: FileReader {
    fn get_uri_string(&self, uuid: Uuid) -> LSPRVDocument;
}

impl CanGetURIString for LSPFileReader {
    fn get_uri_string(&self, uuid: Uuid) -> LSPRVDocument {
        self.file_uris.get(&uuid).unwrap().clone()
    }
}

impl FileReader for LSPFileReader {
    fn get_filename(&self, uuid: uuid::Uuid) -> String {
        self.file_uris.get(&uuid).unwrap().uri.clone()
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
    fn new(docs: Vec<LSPRVDocument>) -> Self {
        let mut map = HashMap::new();

        for doc in docs {
            let uuid = Uuid::new_v4();
            map.insert(uuid, doc);
        }

        LSPFileReader { file_uris: map }
    }
}

#[derive(Deserialize, Clone)]
struct LSPRVDocument {
    uri: String,
    text: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct LSPRVDiagnostic {
    uri: String,
    diagnostics: Vec<Diagnostic>,
}

struct LSPRVSingleDiagnostic {
    uri: String,
    diagnostic: Diagnostic,
}

#[wasm_bindgen]
pub fn riscv_get_diagnostics(docs: JsValue) -> JsValue {
    // convert docs to Vec<LSPRVDocument>
    let docs: Vec<LSPRVDocument> = serde_wasm_bindgen::from_value(docs).unwrap();
    // let mut import_map = HashMap::new();
    let mut imported = HashSet::new();

    // parse and lex all files, without imports and collect that info
    for doc in docs.clone() {
        // let mut imports = HashSet::new();
        let mut parser = RVParser::new(LSPFileReader::new(docs.clone()));
        let items = parser.parse(&doc.uri, true);
        for item in items.0 {
            match item {
                ParserNode::Directive(x) => match x.dir {
                    DirectiveType::Include(name) => {
                        // get full file path
                        let this_uri = parser.get_full_url(&name.data, x.token.file);
                        // add to set
                        imported.insert(this_uri);
                        // imports.insert(this_uri);
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        // import_map.insert(doc.uri, imports);
    }

    // filter out files that are imported by anything
    let to_parse = docs
        .clone()
        .into_iter()
        .filter(|x| !imported.contains(&x.uri));

    // lint errors
    let mut errs = Vec::new();
    // parse and lex all files, with imports
    for doc in to_parse {
        let mut parser = RVParser::new(LSPFileReader::new(docs.clone()));
        let items = parser.parse(&doc.uri, false);

        // add parse errors to errs
        items
            .1
            .iter()
            .map(|x| LSPRVSingleDiagnostic {
                uri: parser.reader.get_filename(x.file()),
                diagnostic: Diagnostic::from(x),
            })
            .for_each(|x| errs.push(x));

        // make CFG
        let cfg = Cfg::new(items.0).unwrap();
        let res = Manager::run(cfg, false);
        match res {
            Ok(new_res) => {
                // add all lint errors to errs
                new_res
                    .iter()
                    .map(|x| LSPRVSingleDiagnostic {
                        uri: parser.reader.get_filename(x.file()),
                        diagnostic: Diagnostic::from(x),
                    })
                    .for_each(|x| errs.push(x));
            }
            Err(e) => return WrapperDiag::new(&format!("{:#?}", e)).into(),
        }
    }

    // insert empty vec for each file
    let mut diag_map = HashMap::new();
    for doc in docs {
        diag_map.insert(doc.uri, Vec::new());
    }

    // collect all diagnostics by file
    for err in errs {
        let uri = err.uri;
        let diag = err.diagnostic;
        let diags = diag_map.entry(uri).or_insert(Vec::new());
        diags.push(diag);
    }

    // convert to Vec<LSPRVDiagnostic>
    let mut errs = Vec::new();
    for (uri, diags) in diag_map {
        errs.push(LSPRVDiagnostic {
            uri: uri,
            diagnostics: diags,
        });
    }

    serde_wasm_bindgen::to_value(&errs).unwrap()
}
