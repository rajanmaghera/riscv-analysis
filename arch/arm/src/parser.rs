use std::fs;
use std::io::Write;
use std::path::{absolute, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use riscv_analysis::parser::{ArithType, IArithType, Imm, JumpLinkRType, JumpLinkType, Label, LabelString, ParserNode, Position, Range, RawToken, Register, Token, With};
use serde::Deserialize;
use uuid::Uuid;

const PARSER: &str = env!("RVA_AARCH64_PARSER");

#[derive(Debug, Deserialize)]
struct InstructionStream {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Deserialize)]
struct Instruction {
    pub opcode: String,
    pub labels: Vec<String>,
    pub operands: Vec<Operand>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value", rename_all="snake_case")]
enum Operand {
    Integer(i64),
    Register(String),
    Label(String),
}

/// Run the parser on a file & return the output.
fn run_parser(path: PathBuf) -> String {
    // Run the parser
    let mut cmd = Command::new(PARSER)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Load the source file
    let abs_path = absolute(path).unwrap();
    let src = fs::read(abs_path).unwrap();

    // Send the source file
    let mut stdin = cmd.stdin.take().unwrap();
    stdin.write_all(&src).unwrap();
    drop(stdin);

    // Get the output
    let output = cmd.wait_with_output().unwrap();
    let out = String::from_utf8(output.stdout)
        .expect("Failed to parse output");
    return out;
}

fn dummy_with<T>(data: T) -> With<T> {
    With {
        token: Token::default(),
        pos: Range::default(),
        file: Uuid::nil(),
        data,
    }
}

fn _make_with<T>(data: T, line: usize, column: usize) -> With<T> {
    With {
        token: Token::default(),
        pos: Range {
            start: Position { line, column, raw_index: 0 },
            end: Position { line, column: column + 1, raw_index: 0 },
        },
        file: Uuid::nil(),
        data,
    }
}

fn make_token(line: usize, column: usize, file_id: Uuid) -> RawToken {
    RawToken {
        text: "".to_string(),
        pos: Range {
            start: Position { line, column, raw_index: 0 },
            end: Position { line, column: column + 1, raw_index: 0 },
        },
        file: file_id,
    }
}

fn map_register(register: &Operand) -> With<Register> {
    let Operand::Register(value) = register else {
        panic!("Instruction error");
    };

    let register = match value.as_str() {
        "W0" => Register::from_str("s0"),
        "W1" => Register::from_str("s1"),
        "W2" => Register::from_str("s2"),
        "W3" => Register::from_str("s3"),
        "W4" => Register::from_str("s4"),
        "W5" => Register::from_str("s5"),
        "SP" => Register::from_str("sp"),
        "LR" => Register::from_str("ra"),
        "WZR" => Register::from_str("zero"),
        e => panic!("Failed to map register: {e}"),
    }.expect("Failed to map register");

    return dummy_with(register);
}

fn map_immediate(imm: &Operand) -> With<Imm> {
    let Operand::Integer(value) = imm else {
        panic!("Instruction error");
    };
    return dummy_with(Imm((*value).try_into().unwrap()));
}

fn map_label(label: &Operand) -> With<LabelString> {
    let Operand::Label(value) = label else {
        panic!("Instruction error");
    };
    return dummy_with(LabelString(value.to_string()));
}

fn label_from_str(label: &str) -> ParserNode {
    let l = Label {
        name: dummy_with(LabelString(label.to_string())),
        key: Uuid::new_v4(),
        token: RawToken::blank(),
    };
    return ParserNode::Label(l);
}

fn each_instruction(inst: &Instruction, file_id: Uuid) -> Vec<ParserNode> {
    let mut acc = vec![];
    for l in inst.labels.iter() {
        acc.push(label_from_str(&l));
    }

    let inst = match inst.opcode.as_str() {
        "ADDWri" => {
            ParserNode::new_iarith(
                dummy_with(IArithType::Addi),
                map_register(&inst.operands[0]),
                map_register(&inst.operands[1]),
                map_immediate(&inst.operands[2]),
                make_token(inst.line, inst.column, file_id),
            )
        },
        "SUBWri" => {
            ParserNode::new_iarith(
                dummy_with(IArithType::Addi),
                map_register(&inst.operands[0]),
                map_register(&inst.operands[1]),
                map_immediate(&inst.operands[2]),
                make_token(inst.line, inst.column, file_id),
            )
        },
        "B" => {
            ParserNode::new_jump_link(
                dummy_with(JumpLinkType::Jal),
                dummy_with(Register::X0),
                map_label(&inst.operands[0]),
                make_token(inst.line, inst.column, file_id),
            )
        },
        "RET" => {
            ParserNode::new_jump_link_r(
                dummy_with(JumpLinkRType::Jalr),
                dummy_with(Register::X0),
                map_register(&inst.operands[0]),
                dummy_with(Imm(0)),
                make_token(inst.line, inst.column, file_id),
            )
        },
        e => panic!("Unknown opcode: {}", e),
    };

    acc.push(inst);
    return acc;
}

fn to_parser_nodes(is: InstructionStream, id: Uuid) -> Vec<ParserNode> {
    let mut acc = vec![];
    acc.push(ParserNode::new_program_entry(id, RawToken::blank()));

    for i in is.instructions {
        for ii in each_instruction(&i, id) {
            acc.push(ii);
        }
    }

    return acc;
}

pub fn parse(path: PathBuf, id: Uuid) -> Vec<ParserNode> {
    let out = run_parser(path);
    let is: InstructionStream = serde_json::from_str(&out).unwrap();
    return to_parser_nodes(is, id);
}
