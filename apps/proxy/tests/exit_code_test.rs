use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_exit_code_success() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.arg("--help")
        .assert()
        .success();
}

#[test]
fn test_exit_code_failure_invalid_arg() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.arg("--invalid-arg")
        .assert()
        .failure();
}

#[test]
fn test_exit_code_no_input() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.assert()
        .failure(); // Should fail when no input provided
}

#[test]
fn test_exit_code_stdin_success() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.arg("-")
        .write_stdin("test content")
        .assert()
        .success();
}
