use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use itertools::Itertools;

use super::{Cfg, CfgNode};

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

        f.write_fmt(format_args!("{}\n", self.node()))?;
        f.write_fmt(format_args!("  | LIVI | {}\n", self.live_in()))?;
        f.write_fmt(format_args!("  | LIVO | {}\n", self.live_out()))?;
        f.write_fmt(format_args!("  | VALO | {}\n", self.reg_values_out()))?;
        f.write_fmt(format_args!("  | STCK | {}\n", self.memory_values_out()))?;
        f.write_fmt(format_args!("  | UDEF | {}\n", self.u_def()))?;
        f.write_fmt(format_args!("  | NEXT | {}\n", self.nexts().len()))?;
        f.write_fmt(format_args!("  | FN   | {fn_label}\n"))?;

        Ok(())
    }
}

impl Display for Cfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in self {
            f.write_fmt(format_args!("{node}\n"))?;
        }
        Ok(())
    }
}
