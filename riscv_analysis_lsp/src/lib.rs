mod lsp;
use lsp::{LSPDiag, LSPFileReader, LSPRVDiagnostic, LSPRVSingleDiagnostic, RVCompletionItem};
use lsp_types::Diagnostic;
use riscv_analysis::parser::{CanGetURIString, ProgramEntryType, RVDocument, RVFileParser};
use riscv_analysis::reader::FileReader;
use serde_wasm_bindgen::to_value;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

// WASM MODULES

#[derive(Default)]
pub struct WrapperDiag(pub Vec<Diagnostic>);

impl From<WrapperDiag> for JsValue {
    fn from(w: WrapperDiag) -> Self {
        to_value(&w.0).unwrap()
    }
}

#[wasm_bindgen]
pub fn riscv_get_uncond_completions() -> JsValue {
    let items = RVCompletionItem::get_all();
    serde_wasm_bindgen::to_value(&items).unwrap()
}

trait FileReading {
    fn get_imports(&mut self, base: &str) -> HashSet<String>;
}

impl<T> FileReading for RVFileParser<T>
where
    T: CanGetURIString + Clone + FileReader,
{
    /// Return the imported files of a file
    fn get_imports(&mut self, base: &str) -> HashSet<String> {
        if let Ok(parse) = self.parse_from_file(base, true) {
            parse
                .include_strings
                .iter()
                .map(|x| x.get_cloned())
                .collect()
        } else {
            HashSet::new()
        }
    }
}

#[wasm_bindgen]
pub fn riscv_get_diagnostics(docs: JsValue) -> JsValue {
    // convert docs to Vec<LSPRVDocument>
    let docs: Vec<RVDocument> = serde_wasm_bindgen::from_value(docs).unwrap();

    // parse and lex all files, without imports and collect that info

    let imported = docs
        .clone()
        .into_iter()
        .map(|doc| RVFileParser::new(LSPFileReader::new(docs.clone())).get_imports(&doc.uri))
        .reduce(|mut x, y| {
            x.extend(y);
            x
        })
        .unwrap_or_default();

    // filter out files that are imported by anything
    let to_parse = docs
        .clone()
        .into_iter()
        .filter(|x| !imported.contains(&x.uri));

    let errs = to_parse
        .flat_map(|f| {
            let mut parser = RVFileParser::new(LSPFileReader::new(docs.clone()));
            let items = parser
                .run(&f.uri, &ProgramEntryType::FirstInstruction)
                .expect("filename not found");
            items
                .into_iter()
                .map(|f| f.to_lsp_diag(&parser))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<LSPRVSingleDiagnostic>>();

    // insert empty vec for each file
    let mut diag_map = docs
        .iter()
        .map(|x| (x.uri.clone(), Vec::new()))
        .collect::<HashMap<_, _>>();

    // collect all diagnostics by fil
    for err in errs {
        let uri = err.uri;
        let diag = err.diagnostic;
        let diags = diag_map.entry(uri).or_insert(Vec::new());
        diags.push(diag);
    }

    let errs = diag_map
        .iter()
        .map(|(uri, diagnostics)| LSPRVDiagnostic {
            uri: uri.clone(),
            diagnostics: diagnostics.clone(),
        })
        .collect::<Vec<_>>();

    serde_wasm_bindgen::to_value(&errs).unwrap()
}
