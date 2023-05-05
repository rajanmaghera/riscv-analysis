use crate::cfg::CFG;
use crate::passes::PassManager;
use lsp_types::{Diagnostic, Position, Range};
use passes::PassErrors;
use serde_wasm_bindgen::to_value;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
mod cfg;
mod lsp;
mod parser;
mod passes;

// WASM MODULES

#[wasm_bindgen]
pub fn riscv_parse(input: &str) -> Result<String, String> {
    Ok(format!("Hello, {}!", input))
}

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

impl Default for WrapperDiag {
    fn default() -> Self {
        WrapperDiag(Vec::new())
    }
}

impl From<PassErrors> for WrapperDiag {
    fn from(e: PassErrors) -> Self {
        let mut diag = Vec::new();
        for err in e.errors {
            diag.push(Diagnostic::from(err));
        }
        WrapperDiag(diag)
    }
}

#[wasm_bindgen]
pub fn riscv_get_diagnostics(input: &str) -> JsValue {
    let cfg = CFG::from_str(input).map_err(|e| format!("{:#?}", e));
    if cfg.is_err() {
        return WrapperDiag::new(&cfg.unwrap_err()).into();
    }
    let res = PassManager::new().run(cfg.unwrap());
    match res {
        Ok(_) => WrapperDiag::default().into(),
        Err(e) => WrapperDiag::from(e).into(),
    }
}
