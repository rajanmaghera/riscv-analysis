use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use itertools::Itertools;

use super::{CFGNode, Cfg};

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

impl Display for CFGNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}\n", self.node()))?;
        f.write_fmt(format_args!("  | LIVI | {}\n", self.live_in().str()))?;
        f.write_fmt(format_args!("  | LIVO | {}\n", self.live_out().str()))?;
        f.write_fmt(format_args!("  | VALO | {}\n", self.reg_values_out().str()))?;
        f.write_fmt(format_args!(
            "  | STCK | {}\n",
            self.stack_values_out().str()
        ))?;
        f.write_fmt(format_args!("  | UDEF | {}\n", self.u_def().str()))?;
        f.write_fmt(format_args!("  | NEXT | {}\n", self.nexts().len()))?;

        Ok(())
    }
}

impl Display for Cfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in &self.nodes {
            f.write_fmt(format_args!("{node}\n"))?;
        }
        Ok(())
    }
}
