use riscv_analysis_cli::wrapper::{DiagnosticTestCase, TestCase};

use std::fs;
use std::iter::zip;
use std::path::{absolute, PathBuf};
use std::env;

use assert_cmd::Command;

fn rva_bin() -> Command {
   Command::cargo_bin("rva").unwrap()
}

fn file_to_path(path: Option<String>) -> PathBuf {
    absolute(PathBuf::from(path.unwrap())).unwrap()
}

fn diagnostic_eq(actual: &DiagnosticTestCase, expected: &DiagnosticTestCase) -> bool {
    // Check if the paths refer to the same file, since they may not have equal
    // content. For example /full/to/a.s and a.s could refer to the same file.
    let actual_path = file_to_path(actual.file.clone());
    let expected_path = file_to_path(expected.file.clone());

    // All other fields must be equal
    return actual.title == expected.title
        && actual_path == expected_path
        && actual.description == expected.description
        && actual.level == expected.level
        && actual.range == expected.range;
}

fn output_eq(actual: TestCase, expected: TestCase) -> bool {
    // There must be the same number of errors
    if actual.diagnostics.len() != expected.diagnostics.len() {
        return false;
    }

    // All diagnostics must be equal
    for (a, e) in zip(actual.diagnostics, expected.diagnostics) {
        if !diagnostic_eq(&a, &e) {
            println!("actual: {:#?}", a);
            println!("expected: {:#?}", e);
            return false;
        }
    }

    true
}

fn run_test(asm: PathBuf, results: PathBuf) {
    // Change the CWD to the directory of this file
    let dir = PathBuf::from("tests/");
    env::set_current_dir(dir).unwrap();

    // Run RVA on the input assembly
    let mut bin = rva_bin();
    let cmd = bin
        .arg("lint")
        .arg("--json")
        .arg(asm);

    // Compare outputs
    let out = String::from_utf8(cmd.output().unwrap().stdout).unwrap();
    let actual: TestCase = serde_json::from_str(&out).unwrap();

    let expected_str = fs::read_to_string(results).unwrap();
    let expected: TestCase = serde_json::from_str(&expected_str).unwrap();

    assert!(output_eq(actual, expected));
}

#[test]
fn no_args() {
    let mut cmd = rva_bin();
    cmd.assert().failure();
}

#[test]
fn sample() {
    let asm = PathBuf::from("./sample/unused-value.s");
    let out = PathBuf::from("./sample/unused-value.json");
    run_test(asm, out);
}
