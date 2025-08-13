use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use super::{Cfg, CfgNode};
use crate::passes::DiagnosticLocation;
use itertools::Itertools;

pub trait SetListString {
    fn str(&self) -> String;
}

impl<T, S> SetListString for HashSet<T, S>
where
    T: Display,
    S: std::hash::BuildHasher,
{
    fn str(&self) -> String {
        self.iter()
            .map(std::string::ToString::to_string)
            .sorted()
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<T, U, S> SetListString for HashMap<T, U, S>
where
    T: Display,
    U: Display,
    S: std::hash::BuildHasher,
{
    fn str(&self) -> String {
        self.iter()
            .map(|(k, v)| format!("[{k}: {v}]"))
            .sorted()
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Display for CfgNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fn_label = match self.functions().len() {
            0 => "N/A".to_string(),
            _ => self
                .functions()
                .iter()
                .map(|func| func.name().to_string())
                .join(" | "),
        };

        f.write_fmt(format_args!(
            "{} -- line {}: \"{}\"\n",
            self.node(),
            self.node().token().range().start().one_idx_line(),
            self.node().token().raw_text()
        ))?;
        f.write_fmt(format_args!("  | LIVI | {}\n", self.live_in()))?;
        f.write_fmt(format_args!("  | LIVO | {}\n", self.live_out()))?;
        f.write_fmt(format_args!("  | VALO | {}\n", self.real_val_out()))?;
        f.write_fmt(format_args!("  | UDEF | {}\n", self.u_def()))?;
        f.write_fmt(format_args!("  | FN   | {fn_label}\n"))?;

        Ok(())
    }
}

impl Display for Cfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in self.iter_source() {
            f.write_fmt(format_args!("{node}\n"))?;
        }
        Ok(())
    }
}
