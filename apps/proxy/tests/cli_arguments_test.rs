use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AI Analytics Proxy Server - Routes AI requests and collects telemetry"));
}

#[test]
fn test_help_short_flag() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.arg("-h");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("AI Analytics Proxy Server - Routes AI requests and collects telemetry"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ai-analytics-proxy"));
}

#[test]
fn test_version_short_flag() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.arg("-v");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ai-analytics-proxy"));
}

#[test]
fn test_usage_flag() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.arg("--usage");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"));
}

#[test]
fn test_no_args_shows_help() {
    let mut cmd = Command::cargo_bin("ai-analytics-proxy").unwrap();
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("AI Analytics Proxy Server - Routes AI requests and collects telemetry"));
}
