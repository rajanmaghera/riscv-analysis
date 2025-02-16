use std::fs;
use std::io::Write;
use std::path::{absolute, PathBuf};
use std::process::{Command, Stdio};
use serde::Deserialize;

const PARSER: &str = env!("RVA_AARCH64_PARSER");

#[derive(Debug, Deserialize)]
pub struct InstructionStream {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Deserialize)]
pub struct Instruction {
    pub opcode: String,
    pub labels: Vec<String>,
    pub operands: Vec<Operand>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value", rename_all="snake_case")]
pub enum Operand {
    Integer(i64),
    Register(String),
    Label(String),
}

/// Run the parser on a file & return the output.
pub fn run_parser(path: PathBuf) -> String {
    // Run the parser
    println!("{}", PARSER);
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

pub fn parse(path: PathBuf) -> InstructionStream {
    let out = run_parser(path);
    let is: InstructionStream = serde_json::from_str(&out).unwrap();
    println!("{:?}", is);

    return InstructionStream { instructions: vec![] };
}
