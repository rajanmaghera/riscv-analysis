#![deny(clippy::all, clippy::pedantic, clippy::cargo)]

use crate::cfg::{AnnotatedCFG, CFG};
use crate::passes::Manager;
use std::str::FromStr;

mod cfg;
mod helpers;
mod parser;
mod passes;
/* This project will start with RV32I exclusively.
 *
 */

fn main() {
    // TODO use rust cli library
    // TODO doctor command that tells you the function definitions, and why it
    // thinks that.

    // Ex. Return values: a0, a1
    // a2 may be a return value, but it is never read from after a function call to __
    // a3 is read from after a function call, but it is not defined in the function
    // check every path of execution to ensure it is assigned a value.
    // You assigned a3 on line xx, but then it is not when line xx (other path) is
    // run. This means that if line xx (join) is run, we cannot guarantee that a3
    // has been assigned in your function.

    // read argument from command line as filename
    // let filename = std::env::args().nth(1).expect("No filename provided");

    let filename = "/Users/rajanmaghera/Documents/GitHub/riscv-analysis/tmp/saved-reg.s";
    let file = std::fs::read_to_string(filename).expect("Unable to read file");

    let cfg = CFG::from_str(file.as_str()).expect("Unable to parse file");
    let acfg = AnnotatedCFG::from(cfg);

    println!("{acfg}");

    let res = Manager::new().run(&acfg);

    for err in res {
        println!("{}({}): {}", err, err.range(), err.long_description());
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::cfg::{VecBlockWrapper, CFG};
    use crate::helpers::{basic_block_from_nodes, tokenize};
    use crate::parser::Imm;
    use crate::parser::Parser;
    use crate::parser::Register;
    use crate::parser::{ArithType, IArithType, LoadType, StoreType};
    use crate::parser::{EqNodeDataVec, Node};
    use crate::parser::{Token, With};
    use crate::passes::Manager;

    // A trait on strings to clean up some code for lexing

    // TODO add extensive support into lexer.
    // This implementation is very basic, just to begin testing files

    #[test]
    fn lex_label() {
        let tokens = tokenize("My_Label:");
        assert_eq!(tokens, vec![Token::Label("My_Label".to_owned())]);
    }

    #[test]
    fn lex_instruction() {
        let tokens = tokenize("add s0, s0, s2");
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("add".to_owned()),
                Token::Symbol("s0".to_owned()),
                Token::Symbol("s0".to_owned()),
                Token::Symbol("s2".to_owned()),
            ]
        );
    }

    #[test]
    fn lex_ints() {
        let tokens = tokenize("0x1234,    0b1010, 1234  -222");
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("0x1234".to_owned()),
                Token::Symbol("0b1010".to_owned()),
                Token::Symbol("1234".to_owned()),
                Token::Symbol("-222".to_owned()),
            ]
        );
    }

    #[test]
    fn lex_long() {
        let tokens = tokenize(
            "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        );
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("add".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x3".into()),
                Token::Newline,
                Token::Label("BLCOK".to_owned()),
                Token::Newline,
                Token::Newline,
                Token::Newline,
                Token::Symbol("sub".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a1".into()),
                Token::Newline,
                Token::Label("my_block".to_owned()),
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
                Token::Newline,
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
            ]
        );
    }

    #[test]
    fn parse_int_from_symbol() {
        assert_eq!(Imm::from_str("1234").unwrap(), Imm(1234));
        assert_eq!(Imm::from_str("-222").unwrap(), Imm(-222));
        assert_eq!(Imm::from_str("0x1234").unwrap(), Imm(4660));
        assert_eq!(Imm::from_str("0b1010").unwrap(), Imm(10));
    }

    #[test]
    fn parse_int_instruction() {
        let parser = Parser::new(
            "addi s0, s0, 0x1234\naddi s0, s0, 0b1010\naddi s0, s0, 1234\naddi s0, s0, -222",
        );
        let ast = parser.collect::<Vec<Node>>();

        assert_eq!(
            vec![
                iarith!(Addi X8 X8 4660),
                iarith!(Addi X8 X8 10),
                iarith!(Addi X8 X8 1234),
                iarith!(Addi X8 X8 -222),
            ]
            .data(),
            ast.data()
        );
    }

    #[test]
    fn parse_instruction() {
        let parser = Parser::new("add s0, s0, s2");
        let ast = parser.collect::<Vec<Node>>();
        assert_eq!(vec![arith!(Add X8 X8 X18)].data(), ast.data());
    }

    #[test]
    fn linear_block() {
        let parser = Parser::new("my_block: add s0, s0, s2\nadd s0, s0, s2\naddi, s1, s1, 0x1");
        let ast = parser.collect::<Vec<Node>>();
        let blocks = CFG::new(ast).expect("unable to create cfg");
        assert_eq!(
            vec![
                basic_block_from_nodes(vec![Node::new_program_entry()]),
                basic_block_from_nodes(vec![
                    arith!(Add X8 X8 X18),
                    arith!(Add X8 X8 X18),
                    iarith!(Addi X9 X9 1),
                ])
            ]
            .data(),
            blocks.blocks.data()
        );
    }

    #[test]
    fn multiple_blocks() {
        let parser = Parser::new(
            "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2\naddi, s1, s1, 0x1",
        );
        let ast = parser.collect::<Vec<Node>>();
        let blocks = CFG::new(ast).expect("unable to create cfg");
        assert_eq!(
            vec![
                basic_block_from_nodes(vec![Node::new_program_entry(), arith!(Add X2 X2 X3),]),
                basic_block_from_nodes(vec![arith!(Sub X10 X10 X11),]),
                basic_block_from_nodes(vec![
                    arith!(Add X8 X8 X18),
                    arith!(Add X8 X8 X18),
                    iarith!(Addi X9 X9 1),
                ])
            ]
            .data(),
            blocks.blocks.data()
        );
    }

    #[test]
    fn block_labels() {
        let blocks = CFG::from_str(
            "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        )
        .expect("unable to create cfg");
        assert_eq!(blocks.labels.len(), 2);
        assert_eq!(
            blocks.labels.get("BLCOK").unwrap(),
            blocks.blocks.get(1).unwrap()
        );
        assert_eq!(
            blocks.labels.get("my_block").unwrap(),
            blocks.blocks.get(2).unwrap()
        );
    }

    #[test]
    fn duplicate_labels() {
        CFG::from_str("my_block: add s0, s0, s2\nmy_block: add s0, s0, s2")
            .expect_err("duplicate labels should fail");
    }

    #[test]
    fn block_labels_with_spaces() {
        let blocks = CFG::from_str(
            "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        )
        .expect("unable to create cfg");
        assert_eq!(blocks.labels.len(), 2);
        assert_eq!(
            blocks.labels.get("BLCOK").unwrap(),
            blocks.blocks.get(1).unwrap()
        );
        assert_eq!(
            blocks.labels.get("my_block").unwrap(),
            blocks.blocks.get(2).unwrap()
        );
    }

    #[test]
    fn lex_comments() {
        let lexer = tokenize(
            "add x2,x2,x3 # hello, world!@#DKSAOKLJu3iou12o\nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        );

        assert_eq!(
            lexer
                .iter()
                .map(|t| t.token.clone())
                .collect::<Vec<Token>>(),
            vec![
                Token::Symbol("add".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x3".into()),
                Token::Newline,
                Token::Label("BLCOK".to_string()),
                Token::Newline,
                Token::Newline,
                Token::Newline,
                Token::Symbol("sub".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a1".into()),
                Token::Newline, // ERROR HERE
                Token::Label("my_block".to_string()),
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
                Token::Newline, // ERROR HERE
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
            ]
        );
    }

    #[test]
    fn parse_bad_memory() {
        let str = "lw x10, 10(x10)\n  lw  x10, 10  (  x10  )  \n lw x10, 10 (x10)\n lw x10, 10(  x10)\n lw x10, 10(x10 )";

        let parser = Parser::new(str);
        let ast = parser.collect::<Vec<Node>>();

        assert_eq!(
            ast.data(),
            vec![
                load!(Lw X10 X10 10),
                load!(Lw X10 X10 10),
                load!(Lw X10 X10 10),
                load!(Lw X10 X10 10),
                load!(Lw X10 X10 10),
            ]
            .data()
        );
    }

    #[test]
    fn basic_imm() {
        let blocks =
            CFG::from_str("\nhello_world:\n    addi x0, x2 12").expect("unable to create cfg");
        assert_eq!(
            vec![
                basic_block_from_nodes(vec![Node::new_program_entry()]),
                basic_block_from_nodes(vec![iarith!(Addi X0 X2 12),])
            ]
            .data(),
            blocks.blocks.data()
        );
        let blocks = AnnotatedCFG::from(blocks);
        let errs = Manager::new().run(&blocks);
        assert_ne!(errs.len(), 0);
    }

    #[test]
    fn pass_with_comments() {
        let blocks = CFG::from_str("\nhello_world:\n    addi x1, x2 12 # yolo\nadd x1, x2 x3")
            .expect("unable to create cfg");
        assert_eq!(
            vec![
                basic_block_from_nodes(vec![Node::new_program_entry()]),
                basic_block_from_nodes(vec![iarith!(Addi X1 X2 12), arith!(Add X1 X2 X3),])
            ]
            .data(),
            blocks.blocks.data()
        );
    }

    #[test]
    fn no_imm_num() {
        let str = "addi    sp, sp, -16 \nsw      ra, (sp)";
        let ast = Parser::new(str).collect::<Vec<Node>>();

        assert_eq!(
            ast.data(),
            vec![iarith!(Addi X2 X2 -16), store!(Sw X2 X1 0),].data()
        );
    }
}
