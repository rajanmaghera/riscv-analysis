use riscv_analysis_cli::wrapper::{DiagnosticTestCase, TestCase};

use std::env;
use std::fs;
use std::iter::zip;
use std::path::{absolute, PathBuf};

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
    actual.title == expected.title
        && actual_path == expected_path
        && actual.description == expected.description
        && actual.level == expected.level
        && actual.range == expected.range
}

fn output_eq(actual: TestCase, expected: TestCase) -> bool {
    // There must be the same number of errors
    if actual.diagnostics.len() != expected.diagnostics.len() {
        println!("actual: {:#?}", actual.diagnostics);
        println!("expected: {:#?}", expected.diagnostics);
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
    let _ = env::set_current_dir(dir);

    // Run RVA on the input assembly
    let mut bin = rva_bin();
    let cmd = bin.arg("lint").arg("--json").arg(asm);

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

#[test]
fn use_before_assignment_in_entry_instruction() {
    // The function's first instruction reads t0, which nothing defined.
    let asm = PathBuf::from("./sample/use-before-assignment-entry.s");
    let out = PathBuf::from("./sample/use-before-assignment-entry.json");
    run_test(asm, out);
}

#[test]
fn spill_one_word_below_sp_is_clobbered_by_call() {
    // ra is spilled to -4(sp) without a frame; the callee may overwrite
    // that word, so the reload does not restore ra.
    let asm = PathBuf::from("./sample/spill-below-sp-call.s");
    let out = PathBuf::from("./sample/spill-below-sp-call.json");
    run_test(asm, out);
}

#[test]
fn return_through_copy_of_ra_checks_callee_saved_registers() {
    // `jr t0` with t0 = entry ra returns to the caller, so s0 must be restored.
    let asm = PathBuf::from("./sample/return-via-copy-of-ra-s0-not-restored.s");
    let out = PathBuf::from("./sample/return-via-copy-of-ra-s0-not-restored.json");
    run_test(asm, out);
}

#[test]
fn return_through_copy_of_ra_returns_value_to_caller() {
    // a0 set before `jr t0` is live in the caller, and no callee saved
    // register is modified.
    let asm = PathBuf::from("./sample/return-via-copy-of-ra.s");
    let out = PathBuf::from("./sample/return-via-copy-of-ra.json");
    run_test(asm, out);
}

#[test]
fn jump_through_register_to_address_is_not_return() {
    // `jr t0` with t0 = the address of a label is an indirect jump, not a return.
    let asm = PathBuf::from("./sample/jr-to-address-not-return.s");
    let out = PathBuf::from("./sample/jr-to-address-not-return.json");
    run_test(asm, out);
}

#[test]
fn plain_ret_checks_callee_saved_registers() {
    let asm = PathBuf::from("./sample/ret-s0-not-restored.s");
    let out = PathBuf::from("./sample/ret-s0-not-restored.json");
    run_test(asm, out);
}
