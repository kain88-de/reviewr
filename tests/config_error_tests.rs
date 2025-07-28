use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_missing_config_file() {
    let dir = tempdir().unwrap();

    // Try to get config when no config file exists
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path").arg(dir.path()).arg("config");

    // Should succeed with default values
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("allowed_domains: []"));
}

#[test]
fn test_invalid_toml_config() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let invalid_configs = vec![
        ("incomplete_section", "[platforms.gerrit\ngerrit_url = "),
        ("invalid_syntax", "allowed_domains = [invalid"),
        ("wrong_type", "allowed_domains = 123"),
        ("missing_quotes", "allowed_domains = [domain.com]"),
        (
            "mixed_types",
            "[platforms.gerrit]\ngerrit_url = 123\nusername = \"valid\"",
        ),
    ];

    for (test_name, invalid_content) in invalid_configs {
        fs::write(&config_path, invalid_content).unwrap();

        let mut cmd = Command::cargo_bin("reviewr").unwrap();
        cmd.timeout(Duration::from_secs(5));
        cmd.arg("--data-path").arg(dir.path()).arg("config");

        let output = cmd.output().unwrap();

        // Should fail gracefully with helpful error message
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Error should mention config format issue
            assert!(
                stderr.contains("config") || stderr.contains("format") || stderr.contains("parse"),
                "Test '{}' should have config-related error message, got: {}",
                test_name,
                stderr
            );
            // Should not expose internal details
            assert!(!stderr.contains("unwrap()"));
            assert!(!stderr.contains("panic"));
        }

        fs::remove_file(&config_path).ok();
    }
}

#[test]
fn test_config_permission_errors() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    // Create a valid config first
    fs::write(&config_path,
        "version = 1\n[platforms]\n[global_settings]\nallowed_domains = [\"example.com\"]\n[ui_preferences]\ndefault_time_period_days = 30\nshow_platform_icons = true\npreferred_platform_order = [\"gerrit\"]\ntheme = \"Default\""
    ).unwrap();

    // Try to set read-only permissions (Unix-specific test)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&config_path).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&config_path, perms).unwrap();

        // Try to modify the config
        let mut cmd = Command::cargo_bin("reviewr").unwrap();
        cmd.timeout(Duration::from_secs(5));
        cmd.arg("--data-path")
            .arg(dir.path())
            .arg("config")
            .arg("set")
            .arg("allowed_domains")
            .arg("newdomain.com");

        let output = cmd.output().unwrap();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Should indicate permission issue
            assert!(
                stderr.contains("permission")
                    || stderr.contains("access")
                    || stderr.contains("write")
                    || stderr.contains("Permission denied"),
                "Should indicate permission error, got: {}",
                stderr
            );
        }

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&config_path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&config_path, perms).unwrap();
    }
}

#[test]
fn test_config_directory_not_writable() {
    let dir = tempdir().unwrap();

    // Make directory read-only (Unix-specific test)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dir.path()).unwrap().permissions();
        perms.set_mode(0o555); // Read-only directory
        fs::set_permissions(dir.path(), perms).unwrap();

        // Try to create/modify config in read-only directory
        let mut cmd = Command::cargo_bin("reviewr").unwrap();
        cmd.timeout(Duration::from_secs(5));
        cmd.arg("--data-path")
            .arg(dir.path())
            .arg("config")
            .arg("set")
            .arg("allowed_domains")
            .arg("example.com");

        let output = cmd.output().unwrap();

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Should handle directory permission error gracefully
            assert!(!stderr.contains("panic"));
            assert!(!stderr.contains("unwrap()"));
        }

        // Restore permissions for cleanup
        let mut perms = fs::metadata(dir.path()).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dir.path(), perms).unwrap();
    }
}

#[test]
fn test_config_with_missing_required_fields() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let incomplete_configs = vec![
        (
            "gerrit_missing_password",
            r#"
version = 1
[platforms.gerrit]
gerrit_url = "https://gerrit.example.com"
username = "testuser"
# missing http_password
"#,
        ),
        (
            "jira_missing_token",
            r#"
version = 1
[platforms.jira]
jira_url = "https://jira.example.com"
username = "testuser"
# missing api_token
"#,
        ),
        (
            "malformed_url",
            r#"
version = 1
[platforms.gerrit]
gerrit_url = "not-a-valid-url"
username = "testuser"
http_password = "password"
"#,
        ),
    ];

    for (test_name, config_content) in incomplete_configs {
        fs::write(&config_path, config_content).unwrap();

        // Try to use the config (this would be tested during review command)
        let mut cmd = Command::cargo_bin("reviewr").unwrap();
        cmd.timeout(Duration::from_secs(5));
        cmd.arg("--data-path").arg(dir.path()).arg("config");

        let output = cmd.output().unwrap();

        // Config loading should still work, but platform usage would fail later
        // This tests that we don't crash on incomplete platform configs
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("panic") && !stderr.contains("unwrap()"),
            "Test '{}' should not panic on incomplete config",
            test_name
        );

        fs::remove_file(&config_path).ok();
    }
}

#[test]
fn test_config_version_handling() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let version_configs = vec![
        (
            "no_version",
            "[global_settings]\nallowed_domains = [\"example.com\"]",
        ),
        (
            "future_version",
            "version = 999\n[global_settings]\nallowed_domains = [\"example.com\"]",
        ),
        (
            "invalid_version",
            "version = \"not_a_number\"\n[global_settings]\nallowed_domains = [\"example.com\"]",
        ),
        (
            "negative_version",
            "version = -1\n[global_settings]\nallowed_domains = [\"example.com\"]",
        ),
    ];

    for (test_name, config_content) in version_configs {
        fs::write(&config_path, config_content).unwrap();

        let mut cmd = Command::cargo_bin("reviewr").unwrap();
        cmd.timeout(Duration::from_secs(5));
        cmd.arg("--data-path").arg(dir.path()).arg("config");

        let output = cmd.output().unwrap();

        // Should handle version issues gracefully
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("panic"),
            "Test '{}' should handle version gracefully, got: {}",
            test_name,
            stderr
        );

        fs::remove_file(&config_path).ok();
    }
}

#[test]
fn test_concurrent_config_access() {
    let dir = tempdir().unwrap();

    // This test simulates potential race conditions
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let dir_path = dir.path().to_path_buf();
            std::thread::spawn(move || {
                let mut cmd = Command::cargo_bin("reviewr").unwrap();
                cmd.timeout(Duration::from_secs(5));
                cmd.arg("--data-path")
                    .arg(&dir_path)
                    .arg("config")
                    .arg("set")
                    .arg("allowed_domains")
                    .arg(format!("domain{}.com", i));

                cmd.output()
            })
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // At least some operations should succeed
    let success_count = results
        .iter()
        .filter(|r| r.as_ref().map_or(false, |output| output.status.success()))
        .count();
    assert!(
        success_count > 0,
        "At least one concurrent config operation should succeed"
    );

    // None should panic
    for (i, result) in results.iter().enumerate() {
        if let Ok(output) = result {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                !stderr.contains("panic"),
                "Concurrent operation {} should not panic: {}",
                i,
                stderr
            );
        }
    }
}

#[test]
fn test_config_backup_and_recovery() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    // Create initial valid config
    let initial_config = r#"
version = 1
[platforms]
[global_settings]
allowed_domains = ["initial.com"]
[ui_preferences]
default_time_period_days = 30
show_platform_icons = true
preferred_platform_order = ["gerrit"]
theme = "Default"
"#;
    fs::write(&config_path, initial_config).unwrap();

    // Verify initial config works
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path")
        .arg(dir.path())
        .arg("config")
        .arg("get")
        .arg("allowed_domains");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("initial.com"));

    // Simulate config corruption (could happen during failed write)
    fs::write(&config_path, "corrupted content [invalid").unwrap();

    // Application should handle corrupted config gracefully
    let mut cmd = Command::cargo_bin("reviewr").unwrap();
    cmd.timeout(Duration::from_secs(5));
    cmd.arg("--data-path").arg(dir.path()).arg("config");

    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should not panic, should provide helpful error
    assert!(!stderr.contains("panic"));
    if !output.status.success() {
        assert!(stderr.contains("config") || stderr.contains("format") || stderr.contains("parse"));
    }
}
