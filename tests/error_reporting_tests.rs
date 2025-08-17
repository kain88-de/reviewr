use assert_cmd::Command;
use predicates::str;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_error_commands_with_no_errors() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("--data-path")
        .arg(temp_dir.path())
        .arg("errors")
        .arg("list");

    cmd.assert()
        .success()
        .stdout(str::contains("No errors found."));
}

#[test]
fn test_error_stats_with_no_errors() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("--data-path")
        .arg(temp_dir.path())
        .arg("errors")
        .arg("stats");

    cmd.assert()
        .success()
        .stdout(str::contains("No error statistics available."));
}

#[test]
fn test_error_export_with_no_errors() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("--data-path")
        .arg(temp_dir.path())
        .arg("errors")
        .arg("export");

    cmd.assert().success().stdout(str::contains("[]"));
}

#[test]
fn test_error_clear_with_no_log() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("--data-path")
        .arg(temp_dir.path())
        .arg("errors")
        .arg("clear");

    cmd.assert()
        .success()
        .stdout(str::contains("No error log file found."));
}

#[test]
fn test_error_help() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("--data-path").arg(temp_dir.path()).arg("errors");

    cmd.assert()
        .success()
        .stdout(str::contains("Available error commands:"))
        .stdout(str::contains("list"))
        .stdout(str::contains("stats"))
        .stdout(str::contains("export"))
        .stdout(str::contains("clear"));
}

#[test]
fn test_error_list_help() {
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("errors").arg("list").arg("--help");

    cmd.assert()
        .success()
        .stdout(str::contains("Show recent errors"))
        .stdout(str::contains("--platform"))
        .stdout(str::contains("--limit"));
}

#[test]
fn test_error_export_help() {
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("errors").arg("export").arg("--help");

    cmd.assert()
        .success()
        .stdout(str::contains("Export errors to JSON"))
        .stdout(str::contains("--platform"))
        .stdout(str::contains("--output"));
}

#[test]
fn test_error_export_to_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_file = temp_dir.path().join("errors.json");

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("--data-path")
        .arg(temp_dir.path())
        .arg("errors")
        .arg("export")
        .arg("--output")
        .arg(&output_file);

    cmd.assert()
        .success()
        .stdout(str::contains("Exported 0 errors to"));

    // Verify file was created and contains empty JSON array
    let contents = fs::read_to_string(&output_file).expect("Should read output file");
    assert_eq!(contents.trim(), "[]");
}

#[test]
fn test_error_list_with_platform_filter() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("--data-path")
        .arg(temp_dir.path())
        .arg("errors")
        .arg("list")
        .arg("--platform")
        .arg("gerrit");

    cmd.assert()
        .success()
        .stdout(str::contains("No errors found."));
}

#[test]
fn test_error_list_with_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.arg("--data-path")
        .arg(temp_dir.path())
        .arg("errors")
        .arg("list")
        .arg("--limit")
        .arg("5");

    cmd.assert()
        .success()
        .stdout(str::contains("No errors found."));
}
