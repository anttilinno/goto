# Testing Patterns

**Analysis Date:** 2026-01-22

## Test Framework

**Runner:**
- `cargo test` (built-in Rust test harness)
- No external test framework (no criterion, proptest, etc.)

**Assertion Library:**
- Standard `assert!()`, `assert_eq!()`, `assert!()` macros
- `matches!()` macro for error type checking: `assert!(matches!(result, Err(DatabaseError::Alias(AliasError::NotFound(_)))))`

**Run Commands:**
```bash
cargo test                 # Run all tests
cargo test --lib          # Run library unit tests only
cargo test --test '*'     # Run integration tests only
cargo test <test_name>    # Run specific test
```

## Test File Organization

**Location:**
- Unit tests co-located in source files with `#[cfg(test)]` blocks
- Integration tests in `tests/integration.rs`

**Naming:**
- Unit test functions: `test_*` prefix in snake_case
- Integration test functions: `test_*` prefix in snake_case
- Module in source: `#[cfg(test)] mod tests { ... }` at end of file

**Structure:**

Unit tests are in the following source files:
- `src/alias.rs` (lines 152-329): ~70 tests for alias validation, tag operations, error messages
- `src/database.rs` (lines 373-799): ~30 tests for database CRUD, persistence, migration
- `src/fuzzy.rs` (lines 159-295): ~15 tests for string matching and similarity scoring
- `tests/integration.rs`: ~5+ integration tests for CLI workflow

## Test Structure

**Suite Organization** (from `src/alias.rs` lines 152-329):
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

**Patterns:**
- Setup in individual test functions (no shared fixtures at test level)
- Helper functions for common setup patterns: `create_test_db()` in `src/database.rs` (lines 379-384)
- Teardown via dropping variables (tempfile cleanup is automatic)
- Assertions grouped at end of test after setup and action

## Mocking

**Framework:**
- Manual test doubles and fixtures (no mockall, mock crate, etc.)
- `tempfile::tempdir()` for temporary database storage in tests

**Patterns** (from `src/database.rs` lines 379-384):
```rust
fn create_test_db() -> (Database, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let path = dir.path().join("aliases");
    let db = Database::load_from_path(&path).unwrap();
    (db, dir)
}
```

Integration tests use `std::process::Command` to invoke the binary:
```rust
fn goto_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_goto-bin"))
}

#[test]
fn test_register_and_navigate() {
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-r", "test", test_dir.to_str().unwrap()]);
    let output = cmd.output().unwrap();
}
```

**What to Mock:**
- Filesystem: Use `tempfile::tempdir()` for isolated test environments
- CLI interaction: Use `std::process::Command::new()` to invoke binary subprocess
- Database: Create fresh instance for each test via `create_test_db()`

**What NOT to Mock:**
- Core business logic (Alias, Database, fuzzy matching) - test directly
- Error handling - use `Result` unwrap/match in tests to verify errors
- Path resolution - use actual temp directories

## Fixtures and Factories

**Test Data** (from `src/database.rs` lines 391-406):
```rust
#[test]
fn test_insert_and_get() {
    let (mut db, _dir) = create_test_db();
    let alias = Alias::new("test", "/tmp/test").unwrap();
    db.insert(alias);

    assert!(!db.is_empty());
    assert_eq!(db.len(), 1);
    assert!(db.contains("test"));

    let retrieved = db.get("test").unwrap();
    assert_eq!(retrieved.name, "test");
    assert_eq!(retrieved.path, "/tmp/test");
}
```

**Factory Pattern** (from `src/alias.rs` lines 157-164):
```rust
#[test]
fn test_new_alias() {
    let alias = Alias::new("projects", "/home/user/projects").unwrap();
    assert_eq!(alias.name, "projects");
    // ... assertions
}
```

**Location:**
- Helper functions defined at module level in test block
- Test-specific imports via `use super::*;` to access private types
- Temporary directories created inline via `tempdir().unwrap()`

## Coverage

**Requirements:**
- No enforced coverage target
- Coverage tool: Not specified in Cargo.toml
- View coverage: Would use `cargo tarpaulin` or `cargo llvm-cov` if installed

**Current Coverage (observed):**
- Core domain models heavily tested: Alias struct ~100% coverage in unit tests
- Database persistence tested: load/save/migrate paths all tested
- Error cases tested explicitly: see `src/alias.rs` lines 224-227 for error variant testing
- Integration tests cover CLI workflows: register, list, navigate, tags

## Test Types

**Unit Tests:**
- Scope: Single function or method in isolation
- Approach: Create minimal required inputs, verify output/state change
- Location: Co-located in source files in `#[cfg(test)]` blocks
- Example: `test_record_use()` in `src/alias.rs` (lines 191-199) verifies use_count and last_used update

**Integration Tests:**
- Scope: Multi-component workflows (CLI -> config -> database -> filesystem)
- Approach: Use subprocess invocation, temp directories, environment variables
- Location: `tests/integration.rs` (lines 1-150+)
- Example: `test_register_and_navigate()` in `tests/integration.rs` (lines 12-49) tests end-to-end registration and navigation

**E2E Tests:**
- Not separate from integration tests
- Same mechanism: subprocess CLI invocation with temp directories

## Common Patterns

**Async Testing:**
- Not applicable; no async code in codebase

**Error Testing** (from `src/database.rs` lines 409-418):
```rust
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

Pattern: Use `matches!()` macro to check error type and structure without unpacking.

**State Mutations** (from `src/database.rs` lines 731-745):
```rust
#[test]
fn test_auto_saves_on_drop() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("aliases");

    {
        let mut db = Database::load_from_path(&path).unwrap();
        let alias = Alias::new("dropped", "/tmp/dropped").unwrap();
        db.insert(alias);
        // Don't call save() - let Drop handle it
    }

    // Reopen and verify it was saved
    let db = Database::load_from_path(&path).unwrap();
    assert!(db.contains("dropped"));
}
```

Pattern: Use scope blocks to trigger `Drop` behavior, then reopen/reload to verify side effects persisted.

**Dirty Flag Testing** (from `src/database.rs` lines 747-780):
```rust
#[test]
fn test_dirty_flag_not_set_on_read() {
    // Create and populate db
    {
        let mut db = Database::load_from_path(&path).unwrap();
        let alias = Alias::new("test", "/tmp/test").unwrap();
        db.insert(alias);
        db.save().unwrap();
    }

    let mtime_before = fs::metadata(&toml_path).unwrap().modified().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));

    {
        let db = Database::load_from_path(&path).unwrap();
        let _ = db.get("test");
        // On drop, should NOT write since no changes were made
    }

    let mtime_after = fs::metadata(&toml_path).unwrap().modified().unwrap();
    assert_eq!(mtime_before, mtime_after);
}
```

Pattern: Verify optimization behavior by comparing file timestamps before/after read operations.

**Integration Test Pattern** (from `tests/integration.rs` lines 12-49):
```rust
#[test]
fn test_register_and_navigate() {
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
    assert!(output.status.success(), "Register failed: {}", ...);
    assert!(String::from_utf8_lossy(&output.stdout).contains("Registered"));

    // Navigate (verify output)
    let mut cmd = goto_bin();
    cmd.env("GOTO_DB", &db_dir);
    cmd.args(["-x", "test"]);

    let output = cmd.output().unwrap();
    assert!(output.status.success(), "Expand failed: {}", ...);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), test_dir.to_str().unwrap());
}
```

Pattern: Setup temp dirs, run subprocess, check exit status and stdout/stderr.

---

*Testing analysis: 2026-01-22*
