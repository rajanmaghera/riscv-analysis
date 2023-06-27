use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use itertools::Itertools;

use super::{CFGNode, Cfg};

trait SetListString {
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
        f.write_fmt(format_args!("{}\n", self.node))?;
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

// use std::fmt::Display;

// use super::CFGNode;

// impl Display for CFGNode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut index = 0;

//         let mut labels = self.labels_for_branch.iter();
//         for block in &self.blocks {
//             f.write_str("+---------\n")?;
//             f.write_str(&format!(
//                 "| LABELS: {:?}, ID: {}\n",
//                 labels.next().unwrap(),
//                 &block.1.as_simple().to_string().get(..8).unwrap_or("")
//             ))?;
//             f.write_str(&format!(
//                 "| PREV: [{}]\n",
//                 self.directions
//                     .get(block)
//                     .unwrap()
//                     .prev
//                     .iter()
//                     .collect::<Vec<_>>()
//                     .iter()
//                     .map(|x| x
//                         .1
//                         .as_simple()
//                         .to_string()
//                         .get(..8)
//                         .unwrap_or("")
//                         .to_string())
//                     .collect::<Vec<_>>()
//                     .join(", ")
//             ))?;

//             f.write_str("| ****\n")?;
//             for node in &block.0 {
//                 f.write_str(&format!(
//                     "| {:>3}: {}\n|  in: {:<20}\n| out: {:<20}\n",
//                     index,
//                     node,
//                     self.liveness
//                         .live_in
//                         .get(index)
//                         .unwrap()
//                         .iter()
//                         .sorted()
//                         .map(std::string::ToString::to_string)
//                         .collect::<Vec<_>>()
//                         .join(", "),
//                     self.liveness
//                         .live_out
//                         .get(index)
//                         .unwrap()
//                         .iter()
//                         .sorted()
//                         .map(std::string::ToString::to_string)
//                         .collect::<Vec<_>>()
//                         .join(", "),
//                 ))?;
//                 f.write_str(&format!(
//                     "| val: {}\n",
//                     self.available
//                         .avail_out
//                         .get(index)
//                         .unwrap()
//                         .iter()
//                         .sorted_by_key(|x| x.0)
//                         .map(|(k, v)| format!("[{k}: {v}]"))
//                         .collect::<Vec<_>>()
//                         .join(", ")
//                 ))?;
//                 f.write_str(&format!(
//                     "| stk: {}\n",
//                     self.available
//                         .stack_out
//                         .get(index)
//                         .unwrap()
//                         .iter()
//                         .sorted_by_key(|x| x.0)
//                         .map(|(k, v)| format!("[{k}: {v}]"))
//                         .collect::<Vec<_>>()
//                         .join(", ")
//                 ))?;
//                 f.write_str(&format!(
//                     "| udf: {}\n",
//                     self.liveness
//                         .uncond_defs
//                         .get(index)
//                         .unwrap()
//                         .iter()
//                         .sorted()
//                         .map(std::string::ToString::to_string)
//                         .collect::<Vec<_>>()
//                         .join(", ")
//                 ))?;
//                 index += 1;
//             }
//             f.write_str("+---------\n")?;
//         }
//         f.write_str("FUNCTION DATA:\n")?;
//         for k in self.label_entry_map.keys() {
//             f.write_str(&format!(
//                 "{}: {} -> {}\n",
//                 k.0,
//                 self.function_args(&k.0)
//                     .unwrap_or(HashSet::new())
//                     .into_iter()
//                     .map(|x| x.to_string())
//                     .collect::<Vec<_>>()
//                     .join(", "),
//                 self.function_rets(&k.0)
//                     .unwrap_or(HashSet::new())
//                     .into_iter()
//                     .map(|x| x.to_string())
//                     .collect::<Vec<_>>()
//                     .join(", ")
//             ))?;
//         }
//         Ok(())
//     }
// }
