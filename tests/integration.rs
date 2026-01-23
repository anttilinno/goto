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
fn test_stack_push_pop_workflow() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Create two directories to navigate between
    let dir_a = temp.path().join("dir_a");
    let dir_b = temp.path().join("dir_b");
    fs::create_dir(&dir_a).unwrap();
    fs::create_dir(&dir_b).unwrap();

    // Register an alias for dir_b
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "myalias", dir_b.to_str().unwrap()]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Register failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Step 1: Push current location and go to alias
    // Note: The push command uses std::env::current_dir() to get the cwd,
    // so we set the current_dir for the child process
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.current_dir(&dir_a); // Set working directory to dir_a
    cmd.args(["-p", "myalias"]);
    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "Push failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(dir_b.to_str().unwrap()),
        "Push should output target path (dir_b), got: {}",
        stdout
    );

    // Step 2: Pop returns the saved location (dir_a)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-o"]); // pop
    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "Pop failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(dir_a.to_str().unwrap()),
        "Pop should return original dir (dir_a), got: {}",
        stdout
    );

    // Step 3: Pop on empty stack fails
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-o"]);
    let output = cmd.output().unwrap();

    assert!(
        !output.status.success(),
        "Pop on empty stack should fail"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "Exit code should be 1 (not found/empty), got: {:?}",
        output.status.code()
    );
}

#[test]
fn test_stack_multiple_push_operations() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Create directories
    let dir_a = temp.path().join("dir_a");
    let dir_b = temp.path().join("dir_b");
    let dir_c = temp.path().join("dir_c");
    fs::create_dir(&dir_a).unwrap();
    fs::create_dir(&dir_b).unwrap();
    fs::create_dir(&dir_c).unwrap();

    // Register aliases
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "aliasb", dir_b.to_str().unwrap()]);
    cmd.output().unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "aliasc", dir_c.to_str().unwrap()]);
    cmd.output().unwrap();

    // Push from dir_a to aliasb (dir_b)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.current_dir(&dir_a);
    cmd.args(["-p", "aliasb"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "First push failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Push from dir_b to aliasc (dir_c)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.current_dir(&dir_b);
    cmd.args(["-p", "aliasc"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Second push failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Pop should return dir_b (last pushed)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-o"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(dir_b.to_str().unwrap()),
        "First pop should return dir_b, got: {}",
        stdout
    );

    // Pop should return dir_a (first pushed)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-o"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(dir_a.to_str().unwrap()),
        "Second pop should return dir_a, got: {}",
        stdout
    );

    // Third pop should fail (stack empty)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-o"]);
    let output = cmd.output().unwrap();
    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_recent_navigation() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Create directories
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    let dir_c = temp.path().join("gamma");
    fs::create_dir_all(&dir_a).unwrap();
    fs::create_dir_all(&dir_b).unwrap();
    fs::create_dir_all(&dir_c).unwrap();

    // Register aliases
    for (name, path) in [("alpha", &dir_a), ("beta", &dir_b), ("gamma", &dir_c)] {
        let mut cmd = goto_bin();
        cmd.env("GOTO_DB", &db_dir);
        cmd.args(["-r", name, path.to_str().unwrap()]);
        assert!(cmd.output().unwrap().status.success());
    }

    // Navigate in order: alpha, beta, gamma
    // Use navigate (not expand) to record usage
    for name in ["alpha", "beta", "gamma"] {
        let mut cmd = goto_bin();
        cmd.env("GOTO_DB", &db_dir);
        cmd.arg(name); // Navigate (records usage)
        assert!(
            cmd.output().unwrap().status.success(),
            "Navigation to {} should succeed",
            name
        );
    }

    // Check recent list - gamma should be most recent
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--recent"]);
    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "Recent list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("gamma"),
        "Recent should contain gamma: {}",
        stdout
    );
    // gamma should appear first (most recent)
    let gamma_pos = stdout.find("gamma");
    let alpha_pos = stdout.find("alpha");
    if let (Some(g), Some(a)) = (gamma_pos, alpha_pos) {
        assert!(g < a, "gamma should appear before alpha (most recent first)");
    }

    // Get specific recent entry: --recent 1 should return most recent (gamma)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--recent", "1"]);
    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "Recent 1 failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(dir_c.to_str().unwrap()),
        "Recent 1 should return gamma's path, got: {}",
        stdout
    );

    // Clear recent history
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--recent-clear"]);
    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "Recent clear failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify cleared
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--recent"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show "No recently visited" message
    assert!(
        stdout.contains("No recently visited") || !stdout.contains("gamma"),
        "Recent should be cleared, got: {}",
        stdout
    );
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

#[test]
fn test_import_strategies() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let dir_a = temp.path().join("original");
    let dir_b = temp.path().join("imported");
    fs::create_dir_all(&dir_a).unwrap();
    fs::create_dir_all(&dir_b).unwrap();

    // Register initial alias
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "myalias", dir_a.to_str().unwrap()]);
    assert!(cmd.output().unwrap().status.success());

    // Create import file with same alias name but different path
    let import_file = temp.path().join("import.toml");
    let import_content = format!(
        r#"[[aliases]]
name = "myalias"
path = "{}"
tags = []
use_count = 5
created_at = "2024-01-01T00:00:00Z"
"#,
        dir_b.display()
    );
    fs::write(&import_file, &import_content).unwrap();

    // Step 1: Test skip strategy (default)
    // Import with skip - should keep original
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--import", import_file.to_str().unwrap()]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Import with skip failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify original path is preserved
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "myalias"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("original"),
        "Skip should preserve original path, got: {}",
        stdout
    );

    // Step 2: Test overwrite strategy
    // Fresh database for overwrite test
    let db_dir2 = temp.path().join("db2");
    fs::create_dir(&db_dir2).unwrap();

    // Register initial alias
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir2);
    cmd.args(["-r", "myalias", dir_a.to_str().unwrap()]);
    assert!(cmd.output().unwrap().status.success());

    // Import with overwrite
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir2);
    cmd.args([
        "--import",
        import_file.to_str().unwrap(),
        "--strategy=overwrite",
    ]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Import with overwrite failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify path was overwritten
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir2);
    cmd.args(["-x", "myalias"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("imported"),
        "Overwrite should replace path, got: {}",
        stdout
    );

    // Step 3: Test rename strategy
    // Fresh database for rename test
    let db_dir3 = temp.path().join("db3");
    fs::create_dir(&db_dir3).unwrap();

    // Register initial alias
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir3);
    cmd.args(["-r", "myalias", dir_a.to_str().unwrap()]);
    assert!(cmd.output().unwrap().status.success());

    // Import with rename
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir3);
    cmd.args([
        "--import",
        import_file.to_str().unwrap(),
        "--strategy=rename",
    ]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Import with rename failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Both aliases should exist
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir3);
    cmd.args(["--list"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("myalias"),
        "Original alias should exist: {}",
        stdout
    );
    // Renamed one should be myalias_2 (based on find_unique_name logic)
    assert!(
        stdout.contains("myalias_2"),
        "Renamed alias should exist as myalias_2: {}",
        stdout
    );

    // Verify original still points to original dir
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir3);
    cmd.args(["-x", "myalias"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("original"),
        "Original alias should point to original dir: {}",
        stdout
    );

    // Verify renamed points to imported dir
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir3);
    cmd.args(["-x", "myalias_2"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("imported"),
        "Renamed alias should point to imported dir: {}",
        stdout
    );
}

#[test]
fn test_tag_management() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let work_dir = temp.path().join("work");
    let personal_dir = temp.path().join("personal");
    fs::create_dir_all(&work_dir).unwrap();
    fs::create_dir_all(&personal_dir).unwrap();

    // Step 2: Register alias with multiple tags (comma-separated)
    // Use --force to skip confirmation for new tags
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args([
        "-r",
        "work",
        work_dir.to_str().unwrap(),
        "--tags=office,coding,daily",
        "--force",
    ]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Register with multiple tags failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify tags in list output
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--list"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("office"),
        "List should show 'office' tag: {}",
        stdout
    );
    assert!(
        stdout.contains("coding"),
        "List should show 'coding' tag: {}",
        stdout
    );
    assert!(
        stdout.contains("daily"),
        "List should show 'daily' tag: {}",
        stdout
    );

    // Step 3: Register another alias without tags
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "personal", personal_dir.to_str().unwrap()]);
    assert!(
        cmd.output().unwrap().status.success(),
        "Register personal alias failed"
    );

    // Add a tag to the alias (use --force for new tag)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tag", "personal", "home", "--force"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Adding tag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify tag was added
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--list"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("home"),
        "List should show 'home' tag after adding: {}",
        stdout
    );

    // Step 4: Remove the 'daily' tag from work
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--untag", "work", "daily"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Untag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify tag was removed by checking --tags output
    // The 'daily' tag should no longer appear since it was only on 'work'
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tags"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("daily"),
        "'daily' tag should be removed from tags list: {}",
        stdout
    );

    // Step 5: List all tags with counts
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tags"]);
    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "List tags failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show remaining tags
    assert!(
        stdout.contains("office"),
        "Tags list should contain 'office': {}",
        stdout
    );
    assert!(
        stdout.contains("coding"),
        "Tags list should contain 'coding': {}",
        stdout
    );
    assert!(
        stdout.contains("home"),
        "Tags list should contain 'home': {}",
        stdout
    );

    // Step 6: Test tag normalization - adding uppercase tag should normalize to lowercase
    // Use --force for new tag
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tag", "personal", "WORK", "--force"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Adding uppercase tag failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify tag was normalized to lowercase
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tags"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The tag should appear as lowercase 'work'
    assert!(
        stdout.contains("work"),
        "Uppercase tag should be normalized to lowercase 'work': {}",
        stdout
    );

    // Test adding duplicate tag (same tag twice) - should not create duplicates
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tag", "work", "office"]); // 'office' already exists on 'work'
    let _output = cmd.output().unwrap();
    // This might succeed (no-op) or fail depending on implementation
    // Let's verify there's only one 'office' tag in the tags list
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tags"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Count occurrences of "office" - should appear exactly once in the tag listing
    let office_count = stdout.matches("office").count();
    assert!(
        office_count == 1,
        "Tag 'office' should appear exactly once in tags list (got {}): {}",
        office_count,
        stdout
    );
}

#[test]
fn test_list_sort_and_filter() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Create directories
    let work = temp.path().join("work");
    let home = temp.path().join("home");
    let proj = temp.path().join("projects");
    fs::create_dir_all(&work).unwrap();
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&proj).unwrap();

    // Register with tags (use --force to skip confirmation for new tags)
    let aliases = [
        ("work", &work, "office"),
        ("home", &home, "personal"),
        ("proj", &proj, "office"),
    ];

    for (name, path, tag) in aliases {
        let mut cmd = goto_bin();
        cmd.env("GOTO_DB", &db_dir);
        cmd.args(["-r", name, path.to_str().unwrap(), &format!("--tags={}", tag), "--force"]);
        assert!(cmd.output().unwrap().status.success());
    }

    // Step 2: Test alphabetical sort (default)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--list", "--sort=alpha"]);
    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // home < proj < work alphabetically
    let home_pos = stdout.find("home").unwrap();
    let proj_pos = stdout.find("proj").unwrap();
    let work_pos = stdout.find("work").unwrap();
    assert!(
        home_pos < proj_pos && proj_pos < work_pos,
        "Expected home < proj < work in alpha sort, got: {}",
        stdout
    );

    // Step 3: Test filter by tag
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--list", "--filter=office"]);
    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show work and proj (tagged office), not home
    assert!(
        stdout.contains("work"),
        "Filtered list should contain 'work': {}",
        stdout
    );
    assert!(
        stdout.contains("proj"),
        "Filtered list should contain 'proj': {}",
        stdout
    );
    // Count occurrences of "home" - should be 0 since it's tagged "personal"
    // Note: "home" might appear in the path, so we check for the alias name pattern
    let home_as_alias = stdout.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("home") && !trimmed.contains("/home")
    });
    assert!(
        !home_as_alias,
        "Filtered list should not contain 'home' alias: {}",
        stdout
    );

    // Step 4: Generate usage for sort testing
    // Navigate to work multiple times to increase usage
    for _ in 0..3 {
        let mut cmd = goto_bin();
        cmd.env("GOTO_DB", &db_dir);
        cmd.arg("work"); // Navigate (records usage)
        assert!(cmd.output().unwrap().status.success());
    }

    // Navigate to proj once
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("proj");
    assert!(cmd.output().unwrap().status.success());

    // Step 5: Test sort by usage
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--list", "--sort=usage"]);
    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // work should come first (most used)
    let work_pos = stdout.find("work").unwrap();
    let proj_pos = stdout.find("proj").unwrap();
    assert!(
        work_pos < proj_pos,
        "work (3 uses) should appear before proj (1 use) in usage sort: {}",
        stdout
    );

    // Step 6: Test sort by recent
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--list", "--sort=recent"]);
    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // proj was most recently accessed
    let proj_pos = stdout.find("proj").unwrap();
    let work_pos = stdout.find("work").unwrap();
    assert!(
        proj_pos < work_pos,
        "proj (most recent) should appear before work in recent sort: {}",
        stdout
    );
}

#[test]
fn test_persistence() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register alias (first invocation)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "persistent", test_dir.to_str().unwrap()]);
    assert!(cmd.output().unwrap().status.success());

    // Completely new command (simulates new terminal session)
    // Verify alias still exists
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "persistent"]);
    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("testdir"));

    // Navigate multiple times to test use_count persistence
    for _ in 0..3 {
        let mut cmd = goto_bin();
        cmd.env("GOTO_DB", &db_dir);
        cmd.args(["-x", "persistent"]);
        assert!(cmd.output().unwrap().status.success());
    }

    // Check stats show the usage
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--stats"]);
    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Stats should reflect usage
    assert!(stdout.contains("persistent") || stdout.len() > 0);
}

#[test]
fn test_cleanup_dry_run() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let valid_dir = temp.path().join("valid");
    let invalid_dir = temp.path().join("will_delete");
    fs::create_dir(&valid_dir).unwrap();
    fs::create_dir(&invalid_dir).unwrap();

    // Register both
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "valid", valid_dir.to_str().unwrap()]);
    assert!(cmd.output().unwrap().status.success());

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "invalid", invalid_dir.to_str().unwrap()]);
    assert!(cmd.output().unwrap().status.success());

    // Delete the directory to make alias invalid
    fs::remove_dir(&invalid_dir).unwrap();

    // Run cleanup with --dry-run
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--cleanup", "--dry-run"]);
    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("invalid")); // Should show what would be removed

    // Verify alias still exists (dry-run didn't delete)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--list"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("invalid")); // Still there
}

#[test]
fn test_alias_name_validation() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("test");
    fs::create_dir(&test_dir).unwrap();

    // Valid names: alphanumeric, hyphens, underscores
    let valid_names = ["my-alias", "my_alias", "alias123", "a"];
    for name in valid_names {
        let mut cmd = goto_bin();
        cmd.env("GOTO_DB", &db_dir);
        cmd.args(["-r", name, test_dir.to_str().unwrap()]);
        assert!(
            cmd.output().unwrap().status.success(),
            "Should accept: {}",
            name
        );

        // Clean up for next test
        let mut cmd = goto_bin();
        cmd.env("GOTO_DB", &db_dir);
        cmd.args(["-u", name]);
        cmd.output().unwrap();
    }

    // Invalid names should fail with exit code 3
    let invalid_names = ["my alias", "alias/path", ""];
    for name in invalid_names {
        let mut cmd = goto_bin();
        cmd.env("GOTO_DB", &db_dir);
        cmd.args(["-r", name, test_dir.to_str().unwrap()]);
        let output = cmd.output().unwrap();
        assert!(!output.status.success(), "Should reject: '{}'", name);
    }
}

#[test]
fn test_long_path() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Create a deeply nested directory
    let mut long_path = temp.path().to_path_buf();
    for i in 0..20 {
        long_path = long_path.join(format!("level{}", i));
    }
    fs::create_dir_all(&long_path).unwrap();

    // Register alias to long path
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "deep", long_path.to_str().unwrap()]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());

    // Verify we can expand it
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "deep"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("level19")); // Deepest level
}

#[test]
fn test_update_command_parsing() {
    // Test that --update and -U are recognized
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // --update should be recognized (will fail gracefully without network)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--update"]);
    let output = cmd.output().unwrap();
    // Should not fail with "unknown option" error
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Unknown option"),
        "--update should be recognized as valid command: {}",
        stderr
    );

    // -U should also work
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-U"]);
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Unknown option"),
        "-U should be recognized as valid command: {}",
        stderr
    );
}

#[test]
fn test_check_update_command() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // --check-update should be recognized
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--check-update"]);
    let output = cmd.output().unwrap();

    // Should not fail with "unknown option" error
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Unknown option"),
        "--check-update should be recognized: {}",
        stderr
    );
}

#[test]
fn test_version_command() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Test -v shows version
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-v"]);
    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("goto version"),
        "Version output should contain 'goto version': {}",
        stdout
    );

    // Should contain version number (e.g., "1.4.0")
    assert!(
        stdout.contains('.'),
        "Version should contain a version number with dots: {}",
        stdout
    );
}

#[test]
fn test_update_cache_file() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Run --check-update which should create/update the cache file
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--check-update"]);
    let _output = cmd.output().unwrap();
    // Don't assert success here as network may not be available

    // After running check-update, the cache file might exist
    // (depends on whether network call succeeded)
    let cache_path = db_dir.join("update_cache.json");

    // If cache exists, verify it's valid JSON
    if cache_path.exists() {
        let content = fs::read_to_string(&cache_path).unwrap();
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(
            parsed.is_ok(),
            "Cache file should be valid JSON: {}",
            content
        );
    }
}

#[test]
fn test_config_shows_update_settings() {
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // --config should show update settings
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--config"]);
    let output = cmd.output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show [update] section
    assert!(
        stdout.contains("[update]"),
        "Config should show [update] section: {}",
        stdout
    );
    assert!(
        stdout.contains("auto_check"),
        "Config should show auto_check setting: {}",
        stdout
    );
    assert!(
        stdout.contains("check_interval_hours"),
        "Config should show check_interval_hours setting: {}",
        stdout
    );
}

// Tests for tag creation confirmation (TAG-01 through TAG-04)

#[test]
fn test_tag_creation_with_force_flag() {
    // TAG-04: --force bypasses all tag confirmation prompts
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register alias with initial tag using --force
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args([
        "-r",
        "proj",
        test_dir.to_str().unwrap(),
        "--tags=existing",
        "--force",
    ]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Register with --force should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Add new tag with --force - should succeed without prompting
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tag", "proj", "newtag", "--force"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Adding tag with --force should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify both tags exist
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--tags");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("existing"),
        "Tag 'existing' should exist: {}",
        stdout
    );
    assert!(
        stdout.contains("newtag"),
        "Tag 'newtag' should exist: {}",
        stdout
    );
}

#[test]
fn test_tag_creation_first_tag_no_confirmation() {
    // TAG-02: First tag (when no tags exist) is created silently
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register alias without tags first
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "proj", test_dir.to_str().unwrap()]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Register should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Add first tag without --force - should succeed (bootstrapping)
    // In non-interactive (test) mode, confirm() returns default (false)
    // but when no tags exist, confirmation is skipped
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tag", "proj", "firsttag"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "First tag should be created without confirmation: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify tag exists
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--tags");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("firsttag"),
        "First tag should be created: {}",
        stdout
    );
}

#[test]
fn test_tag_creation_denied_in_non_interactive() {
    // TAG-03: Non-interactive mode (piped stdin) denies new tag creation by default
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register alias with initial tag using --force
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args([
        "-r",
        "proj",
        test_dir.to_str().unwrap(),
        "--tags=existing",
        "--force",
    ]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Setup should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Try to add new tag without --force in non-interactive mode
    // Should fail because confirm() returns false (default) in non-terminal
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tag", "proj", "newtag"]);
    let output = cmd.output().unwrap();
    assert!(
        !output.status.success(),
        "New tag creation should be denied in non-interactive mode"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cancelled"),
        "Error should mention cancellation: {}",
        stderr
    );
}

#[test]
fn test_register_with_new_tag_denied_in_non_interactive() {
    // TAG-03: Non-interactive denies new tag creation during registration
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let dir1 = temp.path().join("dir1");
    let dir2 = temp.path().join("dir2");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();

    // Register first alias with tag using --force
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args([
        "-r",
        "first",
        dir1.to_str().unwrap(),
        "--tags=existing",
        "--force",
    ]);
    assert!(
        cmd.output().unwrap().status.success(),
        "First registration should succeed"
    );

    // Try to register second alias with new tag without --force
    // Should fail because confirm() returns false in non-terminal
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "second", dir2.to_str().unwrap(), "--tags=newtag"]);
    let output = cmd.output().unwrap();
    assert!(
        !output.status.success(),
        "Registration with new tag should be denied in non-interactive mode"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cancelled"),
        "Error should mention cancellation: {}",
        stderr
    );
}

#[test]
fn test_existing_tag_no_confirmation_needed() {
    // Using a tag that already exists should not prompt for confirmation
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let dir1 = temp.path().join("dir1");
    let dir2 = temp.path().join("dir2");
    fs::create_dir(&dir1).unwrap();
    fs::create_dir(&dir2).unwrap();

    // Register first alias with tag
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args([
        "-r",
        "first",
        dir1.to_str().unwrap(),
        "--tags=work",
        "--force",
    ]);
    assert!(
        cmd.output().unwrap().status.success(),
        "First registration should succeed"
    );

    // Register second alias with same tag WITHOUT --force
    // Should succeed because 'work' already exists
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "second", dir2.to_str().unwrap(), "--tags=work"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Registration with existing tag should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify both aliases have the tag
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--tags");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // 'work' tag should show 2 aliases
    assert!(
        stdout.contains("work") && stdout.contains("2"),
        "Tag 'work' should be on 2 aliases: {}",
        stdout
    );
}

#[test]
fn test_tag_shorthand_force_flag() {
    // Test that -f also works as shorthand for --force
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register alias with initial tag using -f
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args([
        "-r",
        "proj",
        test_dir.to_str().unwrap(),
        "--tags=existing",
        "-f",
    ]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Register with -f should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Add new tag with -f - should succeed
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["--tag", "proj", "newtag", "-f"]);
    let output = cmd.output().unwrap();
    assert!(
        output.status.success(),
        "Adding tag with -f should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify tag exists
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--tags");
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("newtag"),
        "Tag 'newtag' should exist: {}",
        stdout
    );
}

// Tests for multiple fuzzy suggestions (FUZZY-01 through FUZZY-06)

#[test]
fn test_fuzzy_multiple_suggestions_non_interactive() {
    // FUZZY-06: Non-interactive mode (piped stdin) should exit with error, no prompt
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register multiple similar aliases to generate multiple suggestions
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "development", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "developer", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "developing", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Search for typo - non-interactive should fail immediately
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("develpment"); // typo
    let output = cmd.output().unwrap();

    // Should fail (non-interactive cancels)
    assert!(
        !output.status.success(),
        "Non-interactive fuzzy navigation should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cancelled") || stderr.contains("Did you mean"),
        "Expected cancellation or suggestion message, got: {}",
        stderr
    );
}

#[test]
fn test_fuzzy_shows_suggestions_on_stderr() {
    // FUZZY-05: All prompts output to stderr (stdout clean for shell wrapper)
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register similar aliases
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "myproject", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "myprojects", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Search for typo (non-interactive)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("myprojet"); // typo
    let output = cmd.output().unwrap();

    // stdout should be EMPTY (no path output when cancelled)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "stdout should be empty when navigation cancelled, got: {}",
        stdout
    );

    // stderr should contain "Did you mean" header
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Did you mean"),
        "stderr should contain 'Did you mean', got: {}",
        stderr
    );
}

#[test]
fn test_fuzzy_non_interactive_behavior() {
    // FUZZY-03/FUZZY-02: In non-interactive mode, prompt_selection returns immediately
    // so we only see "Did you mean:" and "cancelled" - NOT the numbered options
    // (numbered options with percentages are only shown in interactive terminal mode)
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register aliases
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "testproject", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Search for typo
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("testprojet"); // typo
    let output = cmd.output().unwrap();

    // In non-interactive mode:
    // - "Did you mean:" header is shown
    // - prompt_selection() returns None immediately (no options displayed)
    // - "Navigation cancelled" error occurs
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Did you mean"),
        "stderr should contain 'Did you mean', got: {}",
        stderr
    );
    assert!(
        stderr.contains("cancelled"),
        "stderr should contain 'cancelled' for non-interactive mode, got: {}",
        stderr
    );
    // Exit code should be non-zero
    assert!(
        !output.status.success(),
        "Non-interactive fuzzy should fail"
    );
}

#[test]
fn test_fuzzy_multiple_matches_triggers_prompt() {
    // FUZZY-02/FUZZY-01: Multiple similar aliases trigger the selection prompt
    // In non-interactive mode, this results in cancellation
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register multiple similar aliases
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "project1", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "project2", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "project3", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Search for partial match - high confidence match should trigger prompt
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("project"); // matches all three with high similarity
    let output = cmd.output().unwrap();

    // Should show "Did you mean:" and cancel in non-interactive
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Did you mean"),
        "High similarity matches should trigger suggestion, got: {}",
        stderr
    );
    assert!(
        !output.status.success(),
        "Non-interactive should fail with cancellation"
    );
}

#[test]
fn test_fuzzy_low_confidence_no_prompt() {
    // Test that low-confidence matches don't trigger prompt at all
    let temp = tempdir().unwrap();
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Register an alias with very different name
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "xyz", test_dir.to_str().unwrap()]);
    cmd.output().unwrap();

    // Search for something completely different
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("abcdefghijk"); // no similarity
    let output = cmd.output().unwrap();

    // Should just say "not found" without "Did you mean"
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "Low confidence should report 'not found', got: {}",
        stderr
    );
    assert!(
        !stderr.contains("Did you mean"),
        "Low confidence should NOT trigger prompt, got: {}",
        stderr
    );
}
