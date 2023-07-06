use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use itertools::Itertools;

use super::{CFGNode, Cfg};

pub trait SetListString {
    fn str(&self) -> String;
}

impl<T> SetListString for HashSet<T>
where
    T: Display,
{
    fn str(&self) -> String {
        self.iter()
            .map(|x| x.to_string())
            .sorted()
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<T, U> SetListString for HashMap<T, U>
where
    T: Display,
    U: Display,
{
    fn str(&self) -> String {
        self.iter()
            .map(|(k, v)| format!("[{}: {}]", k, v))
            .sorted()
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Display for CFGNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}\n", self.node()))?;
        f.write_fmt(format_args!("  | LIVE | {}\n", self.live_out().str()))?;
        f.write_fmt(format_args!("  | VALS | {}\n", self.reg_values_out().str()))?;
        f.write_fmt(format_args!(
            "  | STCK | {}\n",
            self.stack_values_out().str()
        ))?;
        f.write_fmt(format_args!("  | UDEF | {}\n", self.u_def().str()))?;

        Ok(())
    }
}

impl Display for Cfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for node in &self.nodes {
            f.write_fmt(format_args!("{}\n", node))?;
        }
        Ok(())
    }
}
