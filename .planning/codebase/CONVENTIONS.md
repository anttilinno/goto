# Coding Conventions

**Analysis Date:** 2026-01-22

## Naming Patterns

**Files:**
- Snake case: `src/alias.rs`, `src/database.rs`, `src/fuzzy.rs`
- Command modules in `src/commands/` follow snake_case: `register.rs`, `import_export.rs`, `navigate.rs`
- Test files: inline `#[cfg(test)]` modules within source files (not separate test directories)
- Integration tests in `tests/integration.rs` follow snake_case naming

**Functions:**
- Snake case for all functions: `validate_alias()`, `find_similar()`, `record_usage()`, `levenshtein_distance()`
- Helper functions (private) use leading underscore rarely; most are public module functions
- Command entry points are simple names: `register()`, `navigate()`, `cleanup()`

**Variables:**
- Snake case: `db`, `config`, `alias`, `use_count`, `last_used`, `temp_dir`, `toml_path`
- Boolean flags use `is_`, `has_`, `should_` prefixes: `is_empty()`, `has_tag()`, `should_save()`
- Counters use `_count` suffix: `use_count`, `match_count`, `office_count`
- Paths use `_path` suffix: `toml_path`, `text_path`, `config_path`, `aliases_path`

**Types:**
- Struct names: PascalCase: `Alias`, `Database`, `Config`, `AliasError`, `DatabaseError`
- Enum names: PascalCase: `Command`, `ImportStrategy`, `ShellType`
- Enum variants: PascalCase with descriptive content: `InvalidAlias { alias, reason }`, `NotFound(String)`
- Type aliases: rarely used; prefer explicit types

**Constants:**
- UPPER_SNAKE_CASE for module constants: `VERSION`, `VALID_ALIAS_PATTERN`, `VALID_TAG_PATTERN`
- Used with `const` and `LazyLock` for static regexes

## Code Style

**Formatting:**
- Default Rust formatting (implicitly follows rustfmt conventions)
- 4-space indentation (Rust standard)
- No explicit `.rustfmt.toml` found; uses Cargo defaults
- Line length appears unconstrained (some lines exceed 100 chars)

**Linting:**
- No explicit `.clippy.toml` or linting configuration found
- Code uses idiomatic Rust patterns suggesting clippy compliance
- No strict linting rules enforced in CI config

**Documentation:**
- Module-level docs: `//! [description]` at file head (e.g., `src/alias.rs`, `src/main.rs`)
- Public item docs: `///` format with brief descriptions
- Example: `/// Validate that an alias name is acceptable`
- Examples: Provided via inline code in integration tests, not doc tests
- No public doc comments on private functions

## Import Organization

**Order:**
1. Standard library imports: `use std::...;` (e.g., `use std::fs;`, `use std::path::{Path, PathBuf};`)
2. External crate imports: `use chrono::...;`, `use serde::...;`, `use thiserror::Error;`
3. Internal crate imports: `use crate::alias::...;`, `use crate::database::Database;`
4. Often grouped by functionality with blank lines between groups

**Path Aliases:**
- No path aliases configured (no `#[path = "..."]` usage)
- Relative imports use `crate::module` format: `use crate::commands;`, `use crate::alias::Alias;`

**Module Imports:**
- Explicit item imports preferred: `use crate::alias::{Alias, AliasError};`
- Star imports used rarely; only when importing multiple items from same module

## Error Handling

**Patterns:**
- Custom error enums with `#[derive(Error, Debug)]` and `thiserror::Error`
- Examples: `AliasError`, `DatabaseError`, `ConfigError`
- Error variants use descriptive enum types:
  ```rust
  #[error("invalid alias '{alias}': {reason}")]
  InvalidAlias { alias: String, reason: String },

  #[error("alias '{0}' not found")]
  NotFound(String),
  ```
- Main error handling uses `Result<T, Box<dyn std::error::Error>>` for command functions
- Error conversion: `?` operator with automatic `From` implementations via `thiserror`
- Error mapping in main: `handle_error()` function maps error strings to exit codes (1-5)

**Exit Codes:**
- Code 1: Not found, stack empty
- Code 2: Directory missing
- Code 3: Invalid input, invalid alias, invalid tag
- Code 4: Already exists
- Code 5: System error, IO error
- Code 0: Success

**Error Messages:**
- User-facing errors via `eprintln!()`: `eprintln!("{}", err);`
- Success messages via `println!()`: `println!("Registered '{}'", name);`
- No panic!() in main binary path; errors propagate as `Result`

## Logging

**Framework:** `println!()` and `eprintln!()` macros (no dedicated logging framework)

**Patterns:**
- Status messages to stdout: `println!("Registered '{}' -> {}", name, path);`
- Errors to stderr: `eprintln!("{}", error_message);`
- Update notifications to stderr: `eprintln!("Update available: {}", version);`
- No structured logging; messages are human-readable strings
- Navigation output to stdout: single path per line (consumed by shell wrapper)

## Comments

**When to Comment:**
- Sparse commenting; code is generally self-documenting through naming
- Comments explain WHY, not WHAT: `// Auto-saves on Drop` in database.rs
- No obvious TODO markers; only FIXME/XXX found in code review sections
- Complex algorithms have explanatory comments: Levenshtein distance calculation in `src/fuzzy.rs`

**JSDoc/TSDoc:**
- Not applicable (Rust uses `///` doc comments, not JSDoc)
- Doc comments limited to public module interfaces
- Minimal ceremonial documentation; focus on essential behavior

## Function Design

**Size:**
- Functions are typically 10-50 lines; largest are ~100 lines (migrate_from_text_format)
- Small focused functions with single responsibility: `validate_alias()`, `levenshtein_distance()`, `similarity()`

**Parameters:**
- Explicit over implicit: functions take references where applicable (`&Database`, `&str`)
- Mutable references for stateful operations: `&mut Database`, `&mut Alias`
- Parameters rarely exceed 4 arguments; complex data bundled into structs

**Return Values:**
- `Result<(), Box<dyn std::error::Error>>` for fallible operations
- `Result<String, DatabaseError>` for typed errors
- `Option<T>` for optional lookups: `db.get()` returns `Option<&Alias>`
- Simple success patterns: `Ok(())` or `Err(...)`

**Example Pattern:**
```rust
pub fn register_with_tags(
    db: &mut Database,
    name: &str,
    path: &str,
    tags: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    validate_alias(name)?;
    let normalized_tags = validate_and_normalize_tags(tags)?;
    // ... operation ...
    println!("Registered '{}'", name);
    Ok(())
}
```

## Module Design

**Exports:**
- Command modules export single public function: `pub fn register()`, `pub fn cleanup()`
- Core modules export types and utility functions: `alias.rs` exports `Alias` struct, validation functions
- Internal implementation details kept private
- No re-exports in `mod.rs`; modules imported directly

**Barrel Files:**
- `src/commands/mod.rs` exists but minimal; lists command modules
- No star exports; explicit imports required
- Example: import command with `use goto::commands::register;`

**Module Organization:**
- One module per concern: `alias.rs` (types), `database.rs` (persistence), `fuzzy.rs` (matching)
- Test modules inline: `#[cfg(test)] mod tests { ... }`
- Commands separated into `src/commands/` directory by operation type

## Validation and Error Patterns

**Input Validation:**
- Regex-based validation in `alias.rs`: `VALID_ALIAS_PATTERN`, `VALID_TAG_PATTERN`
- Validation functions return descriptive `Result<(), AliasError>`
- Example: `validate_alias()` checks empty, pattern match, returns specific error variant

**Type Safety:**
- Enum-based command dispatch in `cli.rs` ensures exhaustiveness
- Result types prevent silent failures; all errors propagate

## Trait Implementations

**Common Patterns:**
- `Debug` derived for visibility in testing/error context
- `Error` via `thiserror` macro for custom error types
- `Drop` for auto-save behavior on database shutdown
- `Serialize`/`Deserialize` for config and database persistence
- No custom `Eq`, `Hash`, `Ord` implementations; derived when needed

---

*Convention analysis: 2026-01-22*
