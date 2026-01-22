# Testing Patterns

**Analysis Date:** 2026-01-22

## Test Framework

**Runner:**
- `cargo test` (Rust built-in testing framework)
- No external test runner; uses standard Rust test harness
- Config: Cargo.toml specifies `[[test]]` section with `tests/integration.rs` path

**Assertion Library:**
- Standard Rust assertions: `assert!()`, `assert_eq!()`, `assert_ne!()`
- No external assertion library; relies on standard macros
- Result-based assertions: `assert!(result.is_ok())`, `assert!(matches!(...))`

**Run Commands:**
```bash
cargo test                     # Run all tests
cargo test --lib              # Run library tests only (unit tests in src/)
cargo test --test integration # Run integration tests only
cargo test <test_name>         # Run specific test by name
cargo test -- --nocapture     # Show println! output during tests
cargo test -- --test-threads=1 # Run tests sequentially
```

## Test File Organization

**Location:**
- **Unit tests:** Inline within source files using `#[cfg(test)]` modules
- **Integration tests:** Separate `tests/integration.rs` file
- Pattern: Source files (`src/alias.rs`, `src/database.rs`, `src/fuzzy.rs`) contain comprehensive unit tests
- Integration tests use compiled binary via `env!("CARGO_BIN_EXE_goto-bin")`

**Naming:**
- Test functions: `test_[behavior]` format: `test_new_alias()`, `test_invalid_name_empty()`
- Test modules: `#[cfg(test)] mod tests { ... }`
- Integration test naming: `test_[feature_workflow]`: `test_register_and_navigate()`, `test_stack_push_pop_workflow()`

**Structure:**
```
src/
├── alias.rs           # ~330 lines (160 lines of tests inline)
├── database.rs        # ~800 lines (425 lines of tests inline)
├── fuzzy.rs           # ~296 lines (136 lines of tests inline)
└── ...

tests/
└── integration.rs     # ~1600 lines (end-to-end CLI tests)
```

## Test Structure

**Suite Organization - Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_alias() {
        let alias = Alias::new("projects", "/home/user/projects").unwrap();
        assert_eq!(alias.name, "projects");
        assert_eq!(alias.path, "/home/user/projects");
        assert!(alias.tags.is_empty());
        assert_eq!(alias.use_count, 0);
        assert!(alias.last_used.is_none());
    }

    #[test]
    fn test_invalid_name_empty() {
        let result = Alias::new("", "/home/user");
        assert!(result.is_err());
    }
}
```

**Suite Organization - Integration Tests:**
```rust
fn goto_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_goto-bin"))
}

#[test]
fn test_register_and_navigate() {
    let temp = tempdir().unwrap();
    let test_dir = temp.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();

    // Setup
    let db_dir = temp.path().join("db");
    fs::create_dir(&db_dir).unwrap();

    // Execute
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "test", test_dir.to_str().unwrap()]);
    let output = cmd.output().unwrap();

    // Assert
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Registered"));
}
```

**Patterns:**
- **Setup phase:** Create temp directories, initialize databases
- **Execute phase:** Call function or command, capture output/result
- **Assert phase:** Verify behavior with assertions
- **Cleanup:** Automatic via `tempdir()` scope on drop

## Mocking

**Framework:** `tempfile::tempdir()` crate for temporary filesystem isolation

**Patterns:**
```rust
use tempfile::tempdir;
use std::fs;

fn create_test_db() -> (Database, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("aliases");
    let db = Database::load_from_path(&path).unwrap();
    (db, dir)  // TempDir dropped after test ends
}

#[test]
fn test_with_temp_db() {
    let (mut db, _dir) = create_test_db();
    let alias = Alias::new("test", "/tmp/test").unwrap();
    db.insert(alias);
    // TempDir auto-cleanup when scope ends
}
```

**Isolation Strategy:**
- Each test uses unique `tempdir()` for database files
- Environment variables set per-test: `cmd.env("GOTO_DB", &db_dir)`
- Command execution in subprocesses prevents state pollution

**What to Mock:**
- Filesystem: Use `tempdir()` for all file operations
- Database: Load fresh from temp paths; no shared state
- Time-dependent behavior: Tests don't rely on wall-clock time (use Utc::now() captured in data)

**What NOT to Mock:**
- Core business logic: Test actual Alias, Database, validation
- Actual command execution: Integration tests spawn real binary
- Error conditions: Test real error paths with invalid inputs

## Fixtures and Factories

**Test Data:**
```rust
fn create_test_db() -> (Database, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("aliases");
    let db = Database::load_from_path(&path).unwrap();
    (db, dir)
}

// Setup helper in integration tests
fn goto_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_goto-bin"))
}
```

**Example Fixture - Database with Data:**
```rust
#[test]
fn test_find_similar() {
    let (mut db, _dir) = create_test_db();

    // Arrange
    db.insert(Alias::new("projects", "/tmp/projects").unwrap());
    db.insert(Alias::new("personal", "/tmp/personal").unwrap());
    db.insert(Alias::new("work", "/tmp/work").unwrap());

    // Act
    let similar = db.find_similar("proj", 0.3);

    // Assert
    assert!(similar.contains(&"projects".to_string()));
}
```

**Location:**
- Helper functions at top of `tests` module in each file
- `create_test_db()` in `database.rs` tests, reused throughout
- Command construction in `tests/integration.rs`: `goto_bin()` helper

## Coverage

**Requirements:** No enforced coverage targets; coverage not configured in CI

**View Coverage:**
```bash
# Not currently set up; would require tarpaulin or llvm-cov
# Manual coverage inspection: grep for `#[cfg(test)]` sections
```

**Coverage Observations:**
- Unit tests: ~90% coverage of public APIs (most functions have dedicated tests)
- `alias.rs`: 25 test cases covering validation, tagging, usage tracking
- `database.rs`: 38 test cases covering CRUD, persistence, migration, metadata
- `fuzzy.rs`: 15 test cases covering distance, similarity, matching
- Integration tests: 35+ end-to-end workflows covering all commands

## Test Types

**Unit Tests:**
- Scope: Individual functions and methods in isolation
- Approach: Direct function calls, assertions on return values
- Example: `test_levenshtein_distance()` verifies algorithm correctness
- Location: `#[cfg(test)]` modules within source files
- Coverage: Core business logic, validation, error conditions

**Integration Tests:**
- Scope: Full CLI workflows with real filesystem, command invocation
- Approach: Spawn binary as subprocess, capture stdout/stderr, verify output and exit codes
- Examples:
  - `test_register_and_navigate()`: Register alias, expand it
  - `test_cleanup()`: Register invalid alias, cleanup, verify removal
  - `test_import_strategies()`: Import with skip/overwrite/rename behaviors
- Location: `tests/integration.rs`
- Coverage: All public commands, error handling, persistence across invocations

**E2E Tests:**
- Framework: Rust subprocess testing via `std::process::Command`
- Not separate from integration tests; integration tests are end-to-end

## Common Patterns

**Async Testing:**
- Not applicable; no async code in codebase
- All operations are synchronous

**Error Testing:**
```rust
#[test]
fn test_invalid_name_empty() {
    let result = Alias::new("", "/home/user");
    assert!(result.is_err());
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
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid alias"));
}

// Pattern: Result matching
#[test]
fn test_add_fails_if_exists() {
    let (mut db, _dir) = create_test_db();
    let alias1 = Alias::new("test", "/tmp/test1").unwrap();
    let alias2 = Alias::new("test", "/tmp/test2").unwrap();

    db.add(alias1).unwrap();
    let result = db.add(alias2);

    assert!(matches!(result, Err(DatabaseError::Alias(AliasError::AlreadyExists(_)))));
}
```

**Filesystem Testing:**
```rust
#[test]
fn test_migration() {
    let dir = tempdir().unwrap();
    let text_path = dir.path().join("aliases");
    let toml_path = dir.path().join("aliases.toml");

    // Write old format
    let mut file = fs::File::create(&text_path).unwrap();
    writeln!(file, "projects /home/user/projects").unwrap();
    writeln!(file, "work /home/user/work").unwrap();
    drop(file);

    // Load triggers migration
    let db = Database::load_from_path(&text_path).unwrap();
    assert_eq!(db.len(), 2);
    assert!(toml_path.exists());
    assert!(dir.path().join("aliases.txt.bak").exists());
}
```

**Persistence Testing:**
```rust
#[test]
fn test_save_and_reload() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("aliases");

    // Write data
    {
        let mut db = Database::load_from_path(&path).unwrap();
        let mut alias = Alias::new("test", "/tmp/test").unwrap();
        alias.add_tag("work");
        alias.use_count = 5;
        db.insert(alias);
        db.save().unwrap();
    }

    // Reload and verify
    let db = Database::load_from_path(&path).unwrap();
    assert_eq!(db.len(), 1);
    let alias = db.get("test").unwrap();
    assert_eq!(alias.use_count, 5);
    assert!(alias.has_tag("work"));
}
```

**Subprocess Output Verification:**
```rust
#[test]
fn test_export_import() {
    // Export from first database
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.arg("--export");
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let export_content = String::from_utf8_lossy(&output.stdout);
    assert!(export_content.contains("test"));

    // Import into second database
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir2);
    cmd.args(["--import", export_file.to_str().unwrap()]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("imported"));
}
```

**Multi-Operation Workflows:**
```rust
#[test]
fn test_stack_multiple_push_operations() {
    // Push from dir_a to aliasb (dir_b)
    let mut cmd = goto_bin();
    cmd.current_dir(&dir_a);
    cmd.args(["-p", "aliasb"]);
    assert!(cmd.output().unwrap().status.success());

    // Push from dir_b to aliasc (dir_c)
    let mut cmd = goto_bin();
    cmd.current_dir(&dir_b);
    cmd.args(["-p", "aliasc"]);
    assert!(cmd.output().unwrap().status.success());

    // Pop should return dir_b (LIFO)
    let mut cmd = goto_bin();
    cmd.args(["-o"]);
    let output = cmd.output().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains(dir_b.to_str().unwrap()));

    // Pop should return dir_a
    let mut cmd = goto_bin();
    cmd.args(["-o"]);
    let output = cmd.output().unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains(dir_a.to_str().unwrap()));

    // Pop on empty stack fails
    let mut cmd = goto_bin();
    cmd.args(["-o"]);
    assert!(!cmd.output().unwrap().status.success());
    assert_eq!(cmd.output().unwrap().status.code(), Some(1));
}
```

## Test Isolation and Cleanup

**Automatic Cleanup:**
- `tempdir()` scope: Files deleted on TempDir drop (automatically at test end)
- No manual cleanup required
- Each test has isolated database directory

**State Between Tests:**
- No shared state; each test creates fresh database in temp directory
- Environment variables isolated per subprocess invocation
- No global mutable state in tests

**Setup/Teardown:**
- Setup: Helper functions like `create_test_db()` or inline temp directory creation
- Teardown: Automatic via Rust scope-based cleanup (Drop trait on TempDir)
- No explicit teardown code needed

---

*Testing analysis: 2026-01-22*
