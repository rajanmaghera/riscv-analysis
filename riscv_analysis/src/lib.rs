#![deny(clippy::all, clippy::pedantic, clippy::cargo)]
#![deny(
    clippy::try_err,
    clippy::string_to_string,
    clippy::string_slice,
    clippy::shadow_unrelated,
    clippy::unseparated_literal_suffix,
    clippy::as_underscore,
    clippy::clone_on_ref_ptr,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::deref_by_slicing,
    clippy::empty_drop,
    clippy::empty_structs_with_brackets,
    clippy::exit,
    clippy::expect_used,
    clippy::let_underscore_must_use
)]
#![deny(
    clippy::panic_in_result_fn,
    clippy::use_debug,
    clippy::todo,
    clippy::indexing_slicing
)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]

pub mod analysis;
pub mod cfg;
pub mod fix;
pub mod gen;
pub mod lints;
pub mod parser;
pub mod passes;
pub mod reader;

// #[test]
// fn parse_int_from_symbol() {
//     assert_eq!(Imm::from_str("1234").unwrap(), Imm(1234));
//     assert_eq!(Imm::from_str("-222").unwrap(), Imm(-222));
//     assert_eq!(Imm::from_str("0x1234").unwrap(), Imm(4660));
//     assert_eq!(Imm::from_str("0b1010").unwrap(), Imm(10));
// }

// #[test]
// fn parse_int_instruction() {
//     let parser = Parser::new(
//         "addi s0, s0, 0x1234\naddi s0, s0, 0b1010\naddi s0, s0, 1234\naddi s0, s0, -222",
//     );
//     let nodes = parser.collect::<Vec<ParserNode>>();

//     assert_eq!(
//         vec![
//             iarith!(Addi X8 X8 4660),
//             iarith!(Addi X8 X8 10),
//             iarith!(Addi X8 X8 1234),
//             iarith!(Addi X8 X8 -222),
//         ]
//         .data(),
//         nodes.data()
//     );
// }

// #[test]
// fn parse_instruction() {
//     let parser = Parser::new("add s0, s0, s2");
//     let nodes = parser.collect::<Vec<ParserNode>>();
//     assert_eq!(vec![arith!(Add X8 X8 X18)].data(), nodes.data());
// }

// #[test]
// fn parse_no_imm_num() {
//     let str = "addi    sp, sp, -16 \nsw      ra, (sp)";
//     let nodes = Parser::new(str).collect::<Vec<ParserNode>>();

//     assert_eq!(
//         nodes.data(),
//         vec![iarith!(Addi X2 X2 -16), store!(Sw X2 X1 0),].data()
//     );
// }
// #[test]
// fn parse_bad_memory() {
//     let str = "lw x10, 10(x10)\n  lw  x10, 10  (  x10  )  \n lw x10, 10 (x10)\n lw x10, 10(  x10)\n lw x10, 10(x10 )";

//     let parser = Parser::new(str);
//     let nodes = parser.collect::<Vec<ParserNode>>();

//     assert_eq!(
//         nodes.data(),
//         vec![
//             load!(Lw X10 X10 10),
//             load!(Lw X10 X10 10),
//             load!(Lw X10 X10 10),
//             load!(Lw X10 X10 10),
//             load!(Lw X10 X10 10),
//         ]
//         .data()
//     );
// }

// #[test]
// fn linear_block() {
//     let parser = Parser::new("my_block: add s0, s0, s2\nadd s0, s0, s2\naddi, s1, s1, 0x1");
//     let nodes = parser.collect::<Vec<ParserNode>>();
//     let blocks = BaseCfg::new(nodes).expect("unable to create cfg");
//     assert_eq!(
//         vec![
//             basic_block_from_nodes(vec![Node::new_program_entry()]),
//             basic_block_from_nodes(vec![
//                 arith!(Add X8 X8 X18),
//                 arith!(Add X8 X8 X18),
//                 iarith!(Addi X9 X9 1),
//             ])
//         ]
//         .data(),
//         blocks.blocks.data()
//     );
// }

// #[test]
// fn multiple_blocks() {
//     let parser = Parser::new(
//         "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2\naddi, s1, s1, 0x1",
//     );
//     let nodes = parser.collect::<Vec<ParserNode>>();
//     let blocks = BaseCfg::new(nodes).expect("unable to create cfg");
//     assert_eq!(
//         vec![
//             basic_block_from_nodes(vec![Node::new_program_entry(), arith!(Add X2 X2 X3),]),
//             basic_block_from_nodes(vec![arith!(Sub X10 X10 X11),]),
//             basic_block_from_nodes(vec![
//                 arith!(Add X8 X8 X18),
//                 arith!(Add X8 X8 X18),
//                 iarith!(Addi X9 X9 1),
//             ])
//         ]
//         .data(),
//         blocks.blocks.data()
//     );
// }

// #[test]
// fn block_labels() {
//     let blocks = BaseCfg::from_str(
//         "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
//     )
//     .expect("unable to create cfg");
//     assert_eq!(blocks.labels.len(), 2);
//     assert_eq!(
//         blocks.labels.get("BLCOK").unwrap(),
//         blocks.blocks.get(1).unwrap()
//     );
//     assert_eq!(
//         blocks.labels.get("my_block").unwrap(),
//         blocks.blocks.get(2).unwrap()
//     );
// }

// #[test]
// fn duplicate_labels() {
//     BaseCfg::from_str("my_block: add s0, s0, s2\nmy_block: add s0, s0, s2")
//         .expect_err("duplicate labels should fail");
// }

// #[test]
// fn block_labels_with_spaces() {
//     let blocks = BaseCfg::from_str(
//         "add x2,x2,x3 \nBLCOK:\n\n\nsub a0 a0 a1\nmy_block: add s0, s0, s2\nadd s0, s0, s2",
//     )
//     .expect("unable to create cfg");
//     assert_eq!(blocks.labels.len(), 2);
//     assert_eq!(
//         blocks.labels.get("BLCOK").unwrap(),
//         blocks.blocks.get(1).unwrap()
//     );
//     assert_eq!(
//         blocks.labels.get("my_block").unwrap(),
//         blocks.blocks.get(2).unwrap()
//     );
// }

// #[test]
// fn basic_imm() {
//     let blocks =
//         BaseCfg::from_str("\nhello_world:\n    addi x0, x2 12").expect("unable to create cfg");
//     assert_eq!(
//         vec![
//             basic_block_from_nodes(vec![Node::new_program_entry()]),
//             basic_block_from_nodes(vec![iarith!(Addi X0 X2 12),])
//         ]
//         .data(),
//         blocks.blocks.data()
//     );
//     let blocks = AnnotatedCfg::from(blocks);
//     let errs = Manager::new().run(&blocks);
//     assert_ne!(errs.len(), 0);
// }

// #[test]
// fn pass_with_comments() {
//     let blocks = BaseCfg::from_str("\nhello_world:\n    addi x1, x2 12 # yolo\nadd x1, x2 x3")
//         .expect("unable to create cfg");
//     assert_eq!(
//         vec![
//             basic_block_from_nodes(vec![Node::new_program_entry()]),
//             basic_block_from_nodes(vec![iarith!(Addi X1 X2 12), arith!(Add X1 X2 X3),])
//         ]
//         .data(),
//         blocks.blocks.data()
//     );
// }
