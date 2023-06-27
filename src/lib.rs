use crate::cfg::Cfg;
use crate::passes::Manager;
use lsp_types::{Diagnostic, Position, Range};
use serde_wasm_bindgen::to_value;
use std::str::FromStr;
use wasm_bindgen::prelude::*;
mod analysis;
mod cfg;
mod gen;
mod lints;
mod lsp;
mod parser;
mod passes;

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

#[wasm_bindgen]
pub fn riscv_get_diagnostics(input: &str) -> JsValue {
    let cfg = Cfg::from_str(input).map_err(|e| format!("{:#?}", e));
    match cfg {
        Ok(cfg) => {
            let res = Manager::run(cfg, false);
            match res {
                Ok(new_res) => {
                    WrapperDiag(new_res.iter().map(|x| x.to_owned().into()).collect()).into()
                }
                Err(e) => WrapperDiag::new(&format!("{:#?}", e)).into(),
            }
        }
        Err(e) => WrapperDiag::new(&e).into(),
    }
}
