use crate::cfg::{
    BasicBlock, BlockDataWrapper, BlockWrapper, VecBlockDataWrapper, VecBlockWrapper, CFG,
};
use crate::parser::ast::{ASTNode, EqNodeDataVec, ToDisplayForVecASTNode};
use crate::parser::inst::{ArithType, IArithType};
use crate::parser::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::parser::register::Register;
use crate::parser::token::ToDisplayForVecToken;
use crate::parser::token::{SymbolData, Token, WithToken};
use crate::passes::PassManager;
use std::rc::Rc;
use std::str::FromStr;

mod cfg;
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
    let filename = std::env::args().nth(1).expect("No filename provided");
    let file = std::fs::read_to_string(filename).expect("Unable to read file");

    // create a new lexer and tokenize the file
    let tokens = Lexer::tokenize(file.as_str());
    println!("{}", tokens.to_display());
    let parser = Parser::new(file.as_str());
    let parser: Vec<ASTNode> = parser.collect();
    println!("{}", parser.to_display());

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

    // to make prototyping easier, use the macro to create AST nodes
    // example macro usage rtype!(Add X0 X1 X2)
    macro_rules! arith {
        ($inst:ident $rd:ident $rs1:ident $rs2:ident) => {
            ASTNode::new_arith(
                WithToken::blank(ArithType::$inst),
                WithToken::blank(Register::$rd),
                WithToken::blank(Register::$rs1),
                WithToken::blank(Register::$rs2),
            )
        };
    }

    macro_rules! iarith {
        ($inst:ident $rd:ident $rs1:ident $imm:expr) => {
            ASTNode::new_iarith(
                WithToken::blank(IArithType::$inst),
                WithToken::blank(Register::$rd),
                WithToken::blank(Register::$rs1),
                WithToken::blank(Imm($imm)),
            )
        };
    }

    use super::*;
    use crate::parser::imm::Imm;

    // A trait on strings to clean up some code for lexing

    // TODO add extensive support into lexer.
    // This implementation is very basic, just to begin testing files

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
            vec![BasicBlock::from_nodes(vec![
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
                BasicBlock::from_nodes(vec![arith!(Add X2 X2 X3),]),
                BasicBlock::from_nodes(vec![arith!(Sub X10 X10 X11),]),
                BasicBlock::from_nodes(vec![
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
        let lexer = Lexer::tokenize(
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
    fn basic_imm() {
        let blocks =
            CFG::from_str("\nhello_world:\n    addi x0, x2 12").expect("unable to create cfg");
        assert_eq!(
            vec![BasicBlock::from_nodes(vec![iarith!(Addi X0 X2 12),])].data(),
            blocks.blocks.data()
        );

        PassManager::new().run(blocks).unwrap_err();
    }

    #[test]
    fn pass_with_comments() {
        let blocks = CFG::from_str("\nhello_world:\n    addi x1, x2 12 # yolo\nadd x1, x2 x3")
            .expect("unable to create cfg");
        assert_eq!(
            vec![BasicBlock::from_nodes(vec![
                iarith!(Addi X1 X2 12),
                arith!(Add X1 X2 X3),
            ])]
            .data(),
            blocks.blocks.data()
        );
        PassManager::new().run(blocks).unwrap();
    }
}
