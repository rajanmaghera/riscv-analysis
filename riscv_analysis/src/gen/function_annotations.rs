use std::{rc::Rc, vec};

use crate::{
    cfg::{Cfg, CfgNode, Function, RegisterSet},
    parser::{Info, InstructionProperties, JumpLinkType, LabelString, ParserNode, Register, With},
    passes::{CfgError, DiagnosticLocation, GenerationPass},
};

struct MarkData {
    pub found: RegisterSet,
    pub instructions: Vec<Rc<CfgNode>>,
    pub returns: Rc<CfgNode>,
}

pub struct FunctionMarkupPass;

impl FunctionMarkupPass {
    fn mark_reachable(
        cfg: &Cfg,
        entry: &Rc<CfgNode>,
        func: &Rc<Function>,
    ) -> Result<MarkData, Box<CfgError>> {
        let mut defs = RegisterSet::new(); // Registers this function writes to
        let mut returns = None; // Return instructions in this function
        let mut instructions = vec![];

        // Traverse the CFG for all nodes reachable from the entry point
        for node in cfg.iter_nexts(Rc::clone(entry)) {
            // Mark the node as being a part of the given function
            instructions.push(Rc::clone(&node));
            node.insert_function(Rc::clone(func));

            // Collect any registers written to by the node
            if let Some(dest) = node.node().writes_to() {
                defs |= dest.data;
            }

            // Collect return instructions
            if node.node().is_return() {
                // Set the newly found return to be an jump to the previously
                // found return.
                if let Some(ref prev_ret) = returns {
                    let found_ret = Rc::clone(&node);

                    // Fix the prevs & nexts of both returns
                    found_ret.clear_nexts();
                    found_ret.insert_next(Rc::clone(prev_ret));
                    prev_ret.insert_prev(Rc::clone(&found_ret));

                    // Convert the found return into a jump
                    let info = Info {
                        token: crate::parser::Token::Symbol("return".to_string()),
                        pos: found_ret.node().range().clone(),
                        file: found_ret.node().file(),
                    };

                    let inst = With::new(JumpLinkType::Jal, info.clone());
                    let rd = With::new(Register::X0, info.clone());
                    let name = With::new(LabelString("__return__".to_string()), info.clone());
                    let new_node =
                        ParserNode::new_jump_link(inst, rd, name, prev_ret.node().token());
                    let _ = found_ret.set_node(new_node);
                }
                // If this is the first return node, save it
                else {
                    returns = Some(Rc::clone(&node));
                }
            }
        }

        if let Some(ret) = returns {
            Ok(MarkData {
                found: defs,
                instructions,
                returns: ret,
            })
        }
        // TODO: Handle functions with no return statements
        else {
            Err(Box::new(CfgError::UnexpectedError))
        }
    }
}

impl GenerationPass for FunctionMarkupPass {
    fn run(cfg: &mut Cfg) -> Result<(), Box<CfgError>> {
        for entry in &cfg.clone() {
            // Skip all nodes that are not entry points
            if !entry.node().is_function_entry() {
                continue;
            }

            // Get the labels for the entry block
            let labels = entry.labels().iter().cloned().collect::<Vec<_>>();

            // Insert a new function into the CFG
            let func = Rc::new(Function::new(
                labels.clone(),
                vec![],
                Rc::clone(&entry),
                Rc::clone(&entry),
            ));

            for label in &labels {
                cfg.insert_function(label.clone(), Rc::clone(&func));
            }

            // Mark all CFG nodes that are reachable from this entry point
            // FIXME: What to do if there is more than one return
            match Self::mark_reachable(cfg, &entry, &Rc::clone(&func)) {
                Ok(data) => {
                    let _ = func.set_defs(data.found);
                    let _ = func.set_nodes(data.instructions);
                    let _ = func.set_exit(data.returns);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::rc::Rc;

    use crate::cfg::{Cfg, Function};
    use crate::parser::RVStringParser;
    use crate::passes::Manager;

    /// Generate the complete CFG from an input string.
    fn gen_cfg(input: &str) -> Cfg {
        let (nodes, error) = RVStringParser::parse_from_text(input);
        assert_eq!(error.len(), 0);
        Manager::gen_full_cfg(nodes).unwrap()
    }

    /// Map string labels to functions.
    fn function_map(cfg: &Cfg) -> HashMap<String, Rc<Function>> {
        let funcs = cfg.functions();
        funcs
            .iter()
            .map(|both| (both.0.data.0.clone(), both.1.clone()))
            .collect()
    }

    /// Get the textual representation of the instructions in a function.
    fn function_tokens(func: &Rc<Function>) -> HashSet<String> {
        let mut nodes = HashSet::new();
        for node in func.nodes().iter() {
            nodes.insert(node.node().token().text);
        }
        nodes
    }

    #[test]
    fn no_function() {
        // There is no JAL to `main`, so it is not a function.
        let input = "\
            main:                       \n\
                li      a0, 1234        \n\
                li      a7, 0           \n\
                ecall                   \n\
                addi    a7, zero, 10    \n\
                ecall                   \n";

        let cfg = gen_cfg(input);
        let funcs = function_map(&cfg);

        // There should be 0 functions
        assert_eq!(funcs.len(), 0);

        // All nodes should have no function annotations
        for node in &cfg {
            assert_eq!(node.functions().len(), 0);
        }
    }

    #[test]
    fn single_function() {
        // Single function: `fn_a`
        let input = "\
            main:                       \n\
                jal     fn_a            \n\
                addi    a7, zero, 10    \n\
                ecall                   \n\
            fn_a:                       \n\
                lw      a1, 0(sp)       \n\
                mul     a0, a0, a1      \n\
                ret                     \n";

        let cfg = gen_cfg(input);
        let funcs = function_map(&cfg);

        // There should only be one function with label `fn_a`
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs["fn_a"].labels().len(), 1);

        // All nodes in the function have the function annotation
        for node in funcs["fn_a"].nodes().iter() {
            assert_eq!(node.functions().len(), 1);
        }

        // The body of the function matches
        // NOTE: At the moment, there is no easy way to order by location, we we
        //       simply check if all instructions are present.
        let nodes = function_tokens(&funcs["fn_a"]);
        assert_eq!(
            nodes,
            HashSet::from([
                "lw a1 0 ( sp )".to_string(),
                "mul a0 a0 a1".to_string(),
                "ret".to_string(),
            ])
        );
    }

    #[test]
    fn many_functions() {
        // Has functions `fn_a`, `fn_b`, & `fn_c`
        let input = "\
            main:                       \n\
                jal     fn_a            \n\
                jal     fn_b            \n\
                jal     fn_c            \n\
                addi    a7, zero, 10    \n\
                ecall                   \n\
            fn_a:                       \n\
                addi    a1, a0, 0       \n\
                ret                     \n\
            fn_b:                       \n\
                addi    a1, a0, 1       \n\
                ret                     \n\
            fn_c:                       \n\
                addi    a1, a0, 2       \n\
                ret                     \n";

        let cfg = gen_cfg(input);
        let funcs = function_map(&cfg);

        // There should be 3 functions with labels `fn_a`, `fn_b`, & `fn_c`
        assert_eq!(funcs.len(), 3);
        let fn_a = function_tokens(&funcs["fn_a"]);
        let fn_b = function_tokens(&funcs["fn_b"]);
        let fn_c = function_tokens(&funcs["fn_c"]);

        // Check that the function bodies match
        assert_eq!(
            fn_a,
            HashSet::from(["addi a1 a0 0".to_string(), "ret".to_string(),])
        );
        assert_eq!(
            fn_b,
            HashSet::from(["addi a1 a0 1".to_string(), "ret".to_string(),])
        );
        assert_eq!(
            fn_c,
            HashSet::from(["addi a1 a0 2".to_string(), "ret".to_string(),])
        );
    }

    #[test]
    fn overlapping_functions() {
        let input = "\
            main:                       \n\
                jal     fn_a            \n\
                jal     fn_b            \n\
                addi    a7, zero, 10    \n\
                ecall                   \n\
            fn_a:                       \n\
                addi    a1, a0, 0       \n\
                addi    a1, a0, 1       \n\
            fn_b:                       \n\
                addi    a1, a0, 2       \n\
                ret                     \n";

        let cfg = gen_cfg(input);
        let funcs = function_map(&cfg);

        // There should be 2 functions with labels `fn_a`, `fn_b`
        assert_eq!(funcs.len(), 2);
        let fn_a = function_tokens(&funcs["fn_a"]);
        let fn_b = function_tokens(&funcs["fn_b"]);

        // Check that the function bodies match
        assert_eq!(
            fn_a,
            HashSet::from([
                // Insructions after label `fn_a` & `fn_b`
                "addi a1 a0 0".to_string(),
                "addi a1 a0 1".to_string(),
                "addi a1 a0 2".to_string(),
                "ret".to_string(),
            ])
        );
        assert_eq!(
            fn_b,
            HashSet::from([
                // Only insructions after label `fn_b`
                "addi a1 a0 2".to_string(),
                "ret".to_string(),
            ])
        );

        // All nodes in `fn_b` should have 2 function annotations
        for nodes in funcs["fn_b"].nodes().iter() {
            assert_eq!(nodes.functions().len(), 2);
        }
    }

    #[test]
    fn interleaved_source() {
        // Two functions have interleaved sources, but share no code
        let input = "\
            main:                       \n\
                jal     fn_a            \n\
                jal     fn_b            \n\
                addi    a7, zero, 10    \n\
                ecall                   \n\
            fn_a:                       \n\
                addi    a1, a0, 0       \n\
                j       fn_a_rest       \n\
            fn_b:                       \n\
                addi    a1, a0, 1       \n\
                ret                     \n\
            fn_a_rest:                  \n\
                addi    a1, a0, 2       \n\
                ret                     \n";

        let cfg = gen_cfg(input);
        let funcs = function_map(&cfg);

        // There should be 2 functions with labels `fn_a`, `fn_b`
        assert_eq!(funcs.len(), 2);
        let fn_a = function_tokens(&funcs["fn_a"]);
        let fn_b = function_tokens(&funcs["fn_b"]);

        // Check that the function bodies match, note that there is no overlap
        assert_eq!(
            fn_a,
            HashSet::from([
                "addi a1 a0 0".to_string(),
                "j fn_a_rest".to_string(),
                "addi a1 a0 2".to_string(),
                "ret".to_string(),
            ])
        );
        assert_eq!(
            fn_b,
            HashSet::from(["addi a1 a0 1".to_string(), "ret".to_string(),])
        );

        // Instructions in both functions should only have a single annotation
        for node in funcs["fn_a"].nodes().iter() {
            assert_eq!(node.functions().len(), 1);
        }
        for node in funcs["fn_b"].nodes().iter() {
            assert_eq!(node.functions().len(), 1);
        }
    }
}
