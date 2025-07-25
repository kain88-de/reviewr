use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_add_employee() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path")
        .arg(dir.path())
        .arg("add")
        .arg("John Doe");
    cmd.write_stdin("Engineer\n");
    cmd.assert().success();
}

#[test]
fn test_config_set_get() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path")
        .arg(dir.path())
        .arg("config")
        .arg("set")
        .arg("allowed_domains")
        .arg("github.com,google.com");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path")
        .arg(dir.path())
        .arg("config")
        .arg("get")
        .arg("allowed_domains");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("github.com"));
}

#[test]
fn test_notes_evidence() {
    let dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path")
        .arg(dir.path())
        .arg("config")
        .arg("set")
        .arg("allowed_domains")
        .arg("localhost");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path")
        .arg(dir.path())
        .arg("add")
        .arg("Jane Doe");
    cmd.write_stdin("Engineer\n");
    cmd.assert().success();

    let mut clipboard = arboard::Clipboard::new().unwrap();
    clipboard
        .set_text("http://localhost:8080/evidence/1")
        .unwrap();

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.env("EDITOR", "true");
    cmd.arg("--data-path")
        .arg(dir.path())
        .arg("notes")
        .arg("Jane Doe");
    cmd.assert().success();

    let notes_path = dir.path().join("notes/Jane Doe.md");
    let notes = fs::read_to_string(notes_path).unwrap();
    assert!(notes.contains("- Evidence: http://localhost:8080/evidence/1"));
}

#[test]
fn test_list_employees() {
    let dir = tempdir().unwrap();

    // Add a few employees first
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path")
        .arg(dir.path())
        .arg("add")
        .arg("Alice Smith");
    cmd.write_stdin("Manager\n");
    cmd.assert().success();

    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path")
        .arg(dir.path())
        .arg("add")
        .arg("Bob Johnson");
    cmd.write_stdin("Developer\n");
    cmd.assert().success();

    // Test list command
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path").arg(dir.path()).arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Employees (2):"))
        .stdout(predicate::str::contains("Alice Smith - Manager"))
        .stdout(predicate::str::contains("Bob Johnson - Developer"));
}

#[test]
fn test_list_empty() {
    let dir = tempdir().unwrap();

    // Test list command with no employees
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path").arg(dir.path()).arg("list");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No employees found."));
}
