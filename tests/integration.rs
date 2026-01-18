//! Integration tests for the goto CLI

use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn goto_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_goto-bin"))
}

#[test]
fn test_register_and_navigate() {
    let temp = tempdir().unwrap();
    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Set custom database path (base directory)
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register alias
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "test", test_dir.to_str().unwrap()]);

    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Register failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Registered"));

    // Navigate (uses -x/expand to just print path without shell CD)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "test"]);

    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Expand failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        test_dir.to_str().unwrap()
    );
}

#[test]
fn test_list_empty() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("-l");

    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "List failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_invalid_alias_name() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "-invalid", "/tmp"]);

    let output = cmd.output().unwrap();
    assert!(
        !output.status.success(),
        "Should fail for invalid alias name"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid alias"),
        "Expected 'invalid alias' error, got: {}",
        stderr
    );
}

#[test]
fn test_fuzzy_suggestion() {
    let temp = tempdir().unwrap();
    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register multiple similar aliases to trigger suggestion mode
    // (single fuzzy match auto-navigates, multiple shows suggestions)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "development", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "developer", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Try with typo - should fail and suggest corrections (multiple matches)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("develpment"); // significant typo

    let output = cmd.output().unwrap();
    assert!(
        !output.status.success(),
        "Should fail for non-existent alias with multiple suggestions"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should give suggestions since there are multiple similar matches
    assert!(
        stderr.contains("Did you mean") || stderr.contains("not found"),
        "Expected fuzzy suggestion, got: {}",
        stderr
    );
}

#[test]
fn test_tags() {
    let temp = tempdir().unwrap();
    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register with tags
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "proj", test_dir.to_str().unwrap(), "--tags=work,rust"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Register with tags failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // List tags
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--tags");
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "List tags failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("work"), "Expected 'work' tag, got: {}", stdout);
    assert!(stdout.contains("rust"), "Expected 'rust' tag, got: {}", stdout);
}

#[test]
fn test_export_import() {
    let temp = tempdir().unwrap();
    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register alias
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "test", test_dir.to_str().unwrap()]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Register failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Export
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--export");
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Export failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let export_content = String::from_utf8_lossy(&output.stdout);
    assert!(
        export_content.contains("test"),
        "Export should contain alias name"
    );

    // Save to file
    let export_file = temp.path().join("export.toml");
    fs::write(&export_file, export_content.as_bytes()).unwrap();

    // Create new database and import
    let db_dir2 = temp.path().join("db2");
    fs::create_dir(&db_dir2).unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir2);
    cmd.args(["--import", export_file.to_str().unwrap()]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Import failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("imported") || stdout.contains("Import"),
        "Expected import confirmation, got: {}",
        stdout
    );

    // Verify alias exists in new database
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir2);
    cmd.args(["-x", "test"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Imported alias should exist: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_unregister() {
    let temp = tempdir().unwrap();
    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register alias
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "todelete", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Unregister
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-u", "todelete"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Unregister failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Unregistered"));

    // Verify alias no longer exists
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "todelete"]);
    let output = cmd.output().unwrap();
    assert!(
        !output.status.success(),
        "Alias should not exist after unregister"
    );
}

#[test]
fn test_cleanup() {
    let temp = tempdir().unwrap();

    // Create a directory that we'll delete later
    let valid_dir = temp.path().join("valid");
    fs::create_dir(&valid_dir).unwrap();

    let invalid_dir = temp.path().join("invalid");
    fs::create_dir(&invalid_dir).unwrap();

    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register both aliases
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "valid", valid_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "invalid", invalid_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Delete the invalid directory
    fs::remove_dir(&invalid_dir).unwrap();

    // Run cleanup
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("-c");
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Cleanup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Valid alias should still exist
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "valid"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Valid alias should still exist: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Invalid alias should be removed
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "invalid"]);
    let output = cmd.output().unwrap();
    assert!(
        !output.status.success(),
        "Invalid alias should be removed after cleanup"
    );
}

#[test]
fn test_show_config() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--config");

    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Show config failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("fuzzy_threshold"),
        "Config should show fuzzy_threshold: {}",
        stdout
    );
    assert!(
        stdout.contains("default_sort"),
        "Config should show default_sort: {}",
        stdout
    );
}

#[test]
fn test_rename_alias() {
    let temp = tempdir().unwrap();
    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register alias
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "oldname", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Rename
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--rename", "oldname", "newname"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Rename failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Old name should not exist
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "oldname"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success(), "Old name should not exist");

    // New name should exist
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "newname"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "New name should exist: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_stats() {
    let temp = tempdir().unwrap();
    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register alias
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "test", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Navigate to record usage (use the alias directly)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("test");
    cmd.output().unwrap();

    // Check stats
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--stats");
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Stats failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_version() {
    let mut cmd = goto_bin();
    cmd.arg("-v");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("version") || stdout.contains("1.0"),
        "Version output: {}",
        stdout
    );
}

#[test]
fn test_help() {
    let mut cmd = goto_bin();
    cmd.arg("-h");

    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"), "Help output: {}", stdout);
    assert!(
        stdout.contains("register") || stdout.contains("-r"),
        "Help output: {}",
        stdout
    );
}

#[test]
fn test_list_with_sort() {
    let temp = tempdir().unwrap();
    let test_dir1 = temp.path().join("dir1");
    let test_dir2 = temp.path().join("dir2");
    fs::create_dir(&test_dir1).unwrap();
    fs::create_dir(&test_dir2).unwrap();

    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register aliases
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "zebra", test_dir1.to_str().unwrap()]);
    cmd.output().unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "alpha", test_dir2.to_str().unwrap()]);
    cmd.output().unwrap();

    // List with alpha sort
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-l", "--sort=alpha"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "List with sort failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Alpha should come before zebra
    let alpha_pos = stdout.find("alpha");
    let zebra_pos = stdout.find("zebra");
    if let (Some(a), Some(z)) = (alpha_pos, zebra_pos) {
        assert!(a < z, "Alpha should come before zebra in alpha sort");
    }
}

#[test]
fn test_tag_and_untag() {
    let temp = tempdir().unwrap();
    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Register alias without tags
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "proj", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Add tag
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tag", "proj", "important"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Tag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify tag exists
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--tags");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("important"),
        "Tag should exist: {}",
        stdout
    );

    // Remove tag
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--untag", "proj", "important"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Untag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
