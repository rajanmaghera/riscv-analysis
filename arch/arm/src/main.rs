use std::path::PathBuf;

mod parser;

fn main() {
    parser::parse(PathBuf::from("./parser/test.aarch64"));
}
