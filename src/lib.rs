use crate::parser::RVParser;
use lsp::{LSPFileReader, LSPRVDiagnostic, LSPRVDocument, LSPRVSingleDiagnostic, RVCompletionItem};
use passes::DebugInfo;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
mod analysis;
mod cfg;
mod gen;
mod lints;
mod lsp;
mod parser;
mod passes;
mod reader;

#[wasm_bindgen]
pub fn riscv_get_uncond_completions() -> JsValue {
    let items = RVCompletionItem::get_all();
    serde_wasm_bindgen::to_value(&items).unwrap()
}

#[wasm_bindgen]
pub fn riscv_get_diagnostics(docs: JsValue) -> JsValue {
    // convert docs to Vec<LSPRVDocument>
    let docs: Vec<LSPRVDocument> = serde_wasm_bindgen::from_value(docs).unwrap();

    // parse and lex all files, without imports and collect that info

    let imported = docs
        .clone()
        .into_iter()
        .map(|doc| RVParser::new(LSPFileReader::new(docs.clone())).get_imports(&doc.uri))
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
            let mut parser = RVParser::new(LSPFileReader::new(docs.clone()));
            let items = parser.run(&f.uri, &DebugInfo::default());
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
