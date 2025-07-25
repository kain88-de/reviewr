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
