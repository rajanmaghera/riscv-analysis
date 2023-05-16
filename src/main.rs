use crate::cfg::CFG;
use crate::passes::{DirectionalCFG, PassManager};
use std::str::FromStr;

mod cfg;
mod helpers;
mod parser;
mod passes;
/* This project will start with RV32I exclusively.
 *
 */

fn main() {
    // Rust uses UTF8 under the hood, but we are going to
    // have a guarantee that only ASCII strings work to
    // simplify some of the logic
    //
    // TODO add check for ASCII strings
    // TODO use rust cli library

    // read argument from command line as filename
    // let filename = std::env::args().nth(1).expect("No filename provided");
    let filename = "/Users/rajanmaghera/sample.s";
    let file = std::fs::read_to_string(filename).expect("Unable to read file");

    // create a new lexer and tokenize the file
    // let tokens = tokenize(file.as_str());
    // println!("{}", tokens.to_display());
    // let parser = Parser::new(file.as_str());
    // let parser: Vec<ASTNode> = parser.collect();
    // println!("{}", parser.to_display());

    let cfg = CFG::from_str(file.as_str()).expect("Unable to parse file");
    let dir = cfg.calculate_directions();
    dir.calculate_in_out();
    println!("\n{}", dir);
    // println!("{:#?}", cfg);
    let res = PassManager::new().run(cfg);

    if res.is_err() {
        println!("Errors found:");
        for err in res.err().unwrap().errors {
            println!("{}({}): {}", err, err.range(), err.long_description());
        }
    } else {
        println!("No errors found");
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::cfg::{VecBlockWrapper, CFG};
    use crate::helpers::*;
    use crate::parser::ast::{ASTNode, EqNodeDataVec};
    use crate::parser::imm::Imm;
    use crate::parser::inst::{ArithType, IArithType, LoadType, StoreType};
    use crate::parser::parser::Parser;
    use crate::parser::register::Register;
    use crate::parser::token::{Token, WithToken};
    use crate::passes::PassManager;

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
        let ast = parser.collect::<Vec<ASTNode>>();

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
        let ast = parser.collect::<Vec<ASTNode>>();
        assert_eq!(vec![arith!(Add X8 X8 X18)].data(), ast.data());
    }

    #[test]
    fn linear_block() {
        let parser = Parser::new("my_block: add s0, s0, s2\nadd s0, s0, s2\naddi, s1, s1, 0x1");
        let ast = parser.collect::<Vec<ASTNode>>();
        let blocks = CFG::new(ast).expect("unable to create cfg");
        assert_eq!(
            vec![basic_block_from_nodes(vec![
                arith!(Add X8 X8 X18),
                arith!(Add X8 X8 X18),
                iarith!(Addi X9 X9 1),
            ])]
            .data(),
            blocks.blocks.data()
        );
    }

    #[test]
    fn multiple_blocks() {
        let parser = Parser::new(
            "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2\naddi, s1, s1, 0x1",
        );
        let ast = parser.collect::<Vec<ASTNode>>();
        let blocks = CFG::new(ast).expect("unable to create cfg");
        assert_eq!(
            vec![
                basic_block_from_nodes(vec![arith!(Add X2 X2 X3),]),
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
        let ast = parser.collect::<Vec<ASTNode>>();

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
            vec![basic_block_from_nodes(vec![iarith!(Addi X0 X2 12),])].data(),
            blocks.blocks.data()
        );

        PassManager::new().run(blocks).unwrap_err();
    }

    #[test]
    fn pass_with_comments() {
        let blocks = CFG::from_str("\nhello_world:\n    addi x1, x2 12 # yolo\nadd x1, x2 x3")
            .expect("unable to create cfg");
        assert_eq!(
            vec![basic_block_from_nodes(vec![
                iarith!(Addi X1 X2 12),
                arith!(Add X1 X2 X3),
            ])]
            .data(),
            blocks.blocks.data()
        );
        PassManager::new().run(blocks).unwrap();
    }

    #[test]
    fn no_imm_num() {
        let str = "addi    sp, sp, -16 \nsw      ra, (sp)";
        let ast = Parser::new(str).collect::<Vec<ASTNode>>();

        assert_eq!(
            ast.data(),
            vec![iarith!(Addi X2 X2 -16), store!(Sw X1 X2 0),].data()
        );
    }
}
