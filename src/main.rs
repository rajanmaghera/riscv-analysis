use crate::cfg::{BasicBlock, CFG};
use crate::parser::ast::{ASTNode, RType};
use crate::parser::lexer::Lexer;
use crate::parser::parser::Parser;
use crate::parser::register::Register;
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
    fn parse_instruction() {
        let parser = Parser::new("add s0, s0, s2");
        let ast = parser.collect::<Vec<WithToken<ASTNode>>>();
        assert_eq!(
            ast,
            vec![WithToken::blank(ASTNode::Add(RType(
                WithToken::blank(Register::X8),
                WithToken::blank(Register::X8),
                WithToken::blank(Register::X18)
            )))]
        );
    }

    #[test]
    fn linear_block() {
        let parser = Parser::new("my_block: add s0, s0, s2\nadd s0, s0, s2");
        let ast = parser.collect::<Vec<WithToken<ASTNode>>>();
        let blocks = CFG::new(ast).expect("unable to create cfg");
        assert_eq!(
            blocks.blocks,
            vec![BasicBlock::from_nodes(vec![
                WithToken::blank(ASTNode::Add(RType(
                    WithToken::blank(Register::X8),
                    WithToken::blank(Register::X8),
                    WithToken::blank(Register::X18)
                ))),
                WithToken::blank(ASTNode::Add(RType(
                    WithToken::blank(Register::X8),
                    WithToken::blank(Register::X8),
                    WithToken::blank(Register::X18)
                ))),
            ])]
        );
    }

    #[test]
    fn multiple_blocks() {
        let parser = Parser::new(
            "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
        );
        let ast = parser.collect::<Vec<WithToken<ASTNode>>>();
        dbg!(&ast);
        let blocks = CFG::new(ast).expect("unable to create cfg");
        assert_eq!(
            blocks.blocks,
            vec![
                BasicBlock::from_nodes(vec![WithToken::blank(ASTNode::Add(RType(
                    WithToken::blank(Register::X2),
                    WithToken::blank(Register::X2),
                    WithToken::blank(Register::X3),
                ))),]),
                BasicBlock::from_nodes(vec![WithToken::blank(ASTNode::Sub(RType(
                    WithToken::blank(Register::X10),
                    WithToken::blank(Register::X10),
                    WithToken::blank(Register::X11),
                ))),]),
                BasicBlock::from_nodes(vec![
                    WithToken::blank(ASTNode::Add(RType(
                        WithToken::blank(Register::X8),
                        WithToken::blank(Register::X8),
                        WithToken::blank(Register::X18)
                    ))),
                    WithToken::blank(ASTNode::Add(RType(
                        WithToken::blank(Register::X8),
                        WithToken::blank(Register::X8),
                        WithToken::blank(Register::X18)
                    ))),
                ])
            ]
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
}
