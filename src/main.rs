use crate::cfg::{BasicBlock, CFG};
use crate::parser::ast::{ASTNode, EqNode, IType, ITypeInst, Imm, RType, RTypeInst};
use crate::parser::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::parser::register::Register;
use crate::parser::token::{SymbolData, Token, WithToken};
use crate::passes::PassManager;
use std::rc::Rc;
use std::str::FromStr;
use crate::parser::token::ToDisplayForVecToken;

mod cfg;
mod parser;
mod passes;

// to make prototyping easier, use the macro to create AST nodes
// example macro usage rtype!(Add X0 X1 X2)
macro_rules! rtype {
    ($inst:ident $rd:ident $rs1:ident $rs2:ident) => {
        ASTNode::new_rtype(
            WithToken::blank(RTypeInst::$inst),
            WithToken::blank(Register::$rd),
            WithToken::blank(Register::$rs1),
            WithToken::blank(Register::$rs2),
        )
    };
}

macro_rules! itype {
    ($inst:ident $rd:ident $rs1:ident $imm:expr) => {
        ASTNode::new_itype(
            WithToken::blank(ITypeInst::$inst),
            WithToken::blank(Register::$rd),
            WithToken::blank(Register::$rs1),
            WithToken::blank(Imm($imm)),
        )
    };
}

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
    let filename = std::env::args().nth(1).expect("No filename provided");
    let file = std::fs::read_to_string(filename).expect("Unable to read file");

    // create a new lexer and tokenize the file
    let tokens = Lexer::tokenize(file.as_str());
    println!("{}", tokens.to_display());

    /*
    let cfg = CFG::from_str(file.as_str()).expect("Unable to parse file");
    println!("{:#?}", cfg);
    let res = PassManager::new().run(cfg);

    if res.is_err() {
        println!("Errors found:");
        for err in res.err().unwrap().errors {
            println!("{}({}): {}", err, err.range(), err.long_description());
        }
    } else {
        println!("No errors found");
    }
    */
}

#[cfg(test)]
mod tests {

    use super::*;

    // A trait on strings to clean up some code for lexing

    // TODO add extensive support into lexer.
    // This implementation is very basic, just to begin testing files

    #[test]
    fn lex_symbols() {
        let input = "   :: , ";
        let tokens = Lexer::tokenize(input);
        assert_eq!(tokens, vec![Token::Colon, Token::Colon, Token::Comma]);
        assert_ne!(tokens, vec![Token::Colon, Token::Comma, Token::Comma]);
    }

    #[test]
    fn lex_label() {
        let tokens = Lexer::tokenize("My_Label:");
        assert_eq!(tokens, vec![Token::Label("My_Label".to_owned())]);
    }

    #[test]
    fn lex_instruction() {
        let tokens = Lexer::tokenize("add s0, s0, s2");
        assert_eq!(
            tokens,
            vec![
                Token::Symbol(SymbolData("add".to_owned())),
                Token::Symbol(SymbolData("s0".to_owned())),
                Token::Symbol(SymbolData("s0".to_owned())),
                Token::Symbol(SymbolData("s2".to_owned())),
            ]
        );
    }

    #[test]
    fn lex_ints() {
        let tokens = Lexer::tokenize("0x1234,    0b1010, 1234  -222");
        assert_eq!(
            tokens,
            vec![
                Token::Symbol(SymbolData("0x1234".to_owned())),
                Token::Symbol(SymbolData("0b1010".to_owned())),
                Token::Symbol(SymbolData("1234".to_owned())),
                Token::Symbol(SymbolData("-222".to_owned())),
            ]
        );
    }

    #[test]
    fn lex_long() {
        let tokens = Lexer::tokenize(
            "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        );
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("add".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x3".into()),
                Token::Label("BLCOK".to_owned()),
                Token::Symbol("sub".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a1".into()),
                Token::Label("my_block".to_owned()),
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
            ]
        );
    }

    #[test]
    fn parse_int_from_symbol() {
        assert_eq!(
            TryInto::<Imm>::try_into(SymbolData("1234".to_owned())).unwrap(),
            Imm(1234)
        );
        assert_eq!(
            TryInto::<Imm>::try_into(SymbolData("-222".to_owned())).unwrap(),
            Imm(-222)
        );
        assert_eq!(
            TryInto::<Imm>::try_into(SymbolData("0x1234".to_owned())).unwrap(),
            Imm(4660)
        );
        assert_eq!(
            TryInto::<Imm>::try_into(SymbolData("0b1010".to_owned())).unwrap(),
            Imm(10)
        );
    }

    #[test]
    fn parse_int_instruction() {
        let parser = Parser::new(
            "addi s0, s0, 0x1234\naddi s0, s0, 0b1010\naddi s0, s0, 1234\naddi s0, s0, -222",
        );
        let ast = parser.collect::<Vec<ASTNode>>();

        assert!(vec![
            itype!(Addi X8 X8 4660),
            itype!(Addi X8 X8 10),
            itype!(Addi X8 X8 1234),
            itype!(Addi X8 X8 -222),
        ]
        .node_eq(&ast));
    }

    #[test]
    fn parse_instruction() {
        let parser = Parser::new("add s0, s0, s2");
        let ast = parser.collect::<Vec<ASTNode>>();
        assert!(vec![rtype!(Add X8 X8 X18)].node_eq(&ast));
    }

    #[test]
    fn linear_block() {
        let parser = Parser::new("my_block: add s0, s0, s2\nadd s0, s0, s2\naddi, s1, s1, 0x1");
        let ast = parser.collect::<Vec<ASTNode>>();
        let blocks = CFG::new(ast).expect("unable to create cfg");
        assert!(vec![BasicBlock::from_nodes(vec![
            rtype!(Add X8 X8 X18),
            rtype!(Add X8 X8 X18),
            itype!(Addi X9 X9 1),
        ])]
        .node_eq(&blocks.blocks));
    }

    #[test]
    fn multiple_blocks() {
        let parser = Parser::new(
            "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2\naddi, s1, s1, 0x1",
        );
        let ast = parser.collect::<Vec<ASTNode>>();
        let blocks = CFG::new(ast).expect("unable to create cfg");
        assert!(vec![
            BasicBlock::from_nodes(vec![rtype!(Add X2 X2 X3),]),
            BasicBlock::from_nodes(vec![rtype!(Sub X10 X10 X11),]),
            BasicBlock::from_nodes(vec![
                rtype!(Add X8 X8 X18),
                rtype!(Add X8 X8 X18),
                itype!(Addi X9 X9 1),
            ])
        ]
        .node_eq(&blocks.blocks));
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
        let lexer = Lexer::tokenize(
            "add x2,x2,x3 # hello, world!@#DKSAOKLJu3iou12o\nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        );

        assert_eq!(
            lexer,
            vec![
                Token::Symbol("add".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x2".into()),
                Token::Symbol("x3".into()),
                Token::Label("BLCOK".to_string()),
                Token::Symbol("sub".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a0".into()),
                Token::Symbol("a1".into()),
                Token::Label("my_block".to_string()),
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
                Token::Symbol("add".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s0".into()),
                Token::Symbol("s2".into()),
            ]
        );
    }

    #[test]
    fn basic_imm() {
        let blocks =
            CFG::from_str("\nhello_world:\n    addi x0, x2 12").expect("unable to create cfg");
        assert!(vec![BasicBlock::from_nodes(vec![itype!(Addi X0 X2 12),])].node_eq(&blocks.blocks));

        PassManager::new().run(blocks).unwrap_err();
    }

    #[test]
    fn pass_with_comments() {
        let blocks = CFG::from_str("\nhello_world:\n    addi x1, x2 12 # yolo\nadd x1, x2 x3")
            .expect("unable to create cfg");
        assert!(vec![BasicBlock::from_nodes(vec![
            itype!(Addi X1 X2 12),
            rtype!(Add X1 X2 X3),
        ])]
        .node_eq(&blocks.blocks));
        PassManager::new().run(blocks).unwrap();
    }
}
