# Coding Conventions

**Analysis Date:** 2026-01-22

## Naming Patterns

**Files:**
- Lowercase with underscores: `alias.rs`, `database.rs`, `fuzzy.rs`
- Command modules in `src/commands/` named after command function: `register.rs`, `navigate.rs`, `cleanup.rs`, `import_export.rs`
- Test modules co-located in same file with `#[cfg(test)]` blocks

**Functions:**
- snake_case throughout: `levenshtein_distance()`, `validate_alias()`, `record_usage()`, `register_with_tags()`
- Action verbs for side-effecting functions: `validate_*`, `record_*`, `navigate()`, `expand()`
- Query/read functions don't have `get_` prefix: `all()`, `contains()`, `names()`, `is_empty()`
- Mutable operations mark parameters explicitly: `&mut Database`, `&mut db`

**Variables:**
- snake_case: `db`, `alias`, `path_str`, `use_count`, `last_used`, `created_at`
- Iteration uses underscore for unused: `_dir` in test fixtures (see `src/database.rs` line 383)
- Field abbreviations minimized: `db` for database is acceptable

**Types:**
- PascalCase for struct/enum names: `Alias`, `Database`, `Command`, `AliasError`, `DatabaseError`
- Error types end with `Error`: `AliasError`, `DatabaseError`, `ConfigError`
- Enum variants PascalCase: `InvalidAlias`, `NotFound`, `AlreadyExists`, `DirectoryNotFound`

## Code Style

**Formatting:**
- Edition 2021 Rust with `cargo fmt` standard formatting
- No explicit formatter configuration detected; defaults apply
- Line length appears to follow standard 100-char Rust conventions

**Linting:**
- No clippy config detected; using default Rust lints
- `#![allow(...)]` not observed in codebase
- No warning suppressions found

## Import Organization

**Order:**
1. `use std::` standard library imports
2. External crate imports (serde, chrono, toml, thiserror, etc.)
3. Local crate imports (self::, crate::)

Example from `src/database.rs` (lines 3-13):
```rust
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::alias::{Alias, AliasError};
use crate::config::{Config, ConfigError};
use crate::fuzzy;
```

**Path Aliases:**
- None observed. Uses full crate paths: `crate::alias`, `crate::config`, `crate::database`
- Module re-exports in `lib.rs` (lines 14-18) for public API: `pub use alias::Alias; pub use cli::{parse_args, Args, Command};`

## Error Handling

**Patterns:**
- Custom error enums using `thiserror` for domain errors: `AliasError`, `DatabaseError`, `ConfigError` (see `src/alias.rs` lines 15-32)
- `Result<T, Box<dyn std::error::Error>>` for operation functions (all command functions in `src/commands/`)
- `Result<T, CustomError>` for module-internal operations (e.g., `Database::save()` returns `Result<(), DatabaseError>`)
- Error conversion via `From` implementations provided by `#[from]` attributes on error enums
- Commands that don't need config/database are handled early and returned (see `src/main.rs` lines 31-70)

Example error handling in `src/commands/register.rs` (lines 21-34):
```rust
validate_alias(name)?;
let normalized_tags = validate_and_normalize_tags(tags)?;
let expanded_path = expand_path(path)?;
let path_str = expanded_path.to_string_lossy().to_string();

if !expanded_path.exists() {
    return Err(AliasError::DirectoryNotFound(path_str).into());
}
```

**Exit Codes:**
- Mapped in `src/main.rs` `handle_error()` function (lines 216-232):
  - 1: Not found or stack empty
  - 2: Directory missing
  - 3: Invalid input (alias/tag)
  - 4: Already exists
  - 5: System/IO error

**Validation Pattern:**
- Separate `validate_*()` functions that return `Result<(), AliasError>` (see `src/alias.rs` lines 35-70)
- Uses lazy-initialized regex patterns with `LazyLock` (lines 9-13): `static VALID_ALIAS_PATTERN: LazyLock<Regex>`
- Validation happens before state modification

## Logging

**Framework:** console (println!, eprintln!)

**Patterns:**
- User messages to stdout via `println!()`
- Errors and diagnostic info to stderr via `eprintln!()`
- Success feedback includes context: `println!("Registered '{}' -> {}", name, path_str);` (src/commands/register.rs line 60)
- No logging framework (tracing/log) used; direct I/O only

## Comments

**When to Comment:**
- Module-level doc comments with `//!` describing purpose (all source files have these)
- Function doc comments with `///` for public APIs (see `src/database.rs` lines 55-58)
- Inline comments rare; code is generally self-documenting
- TODO/FIXME comments not found in codebase

**JSDoc/TSDoc:**
- Rust uses `///` for documentation comments above functions
- Example from `src/alias.rs` (lines 94-95):
```rust
/// Create a new alias with the given name and path
pub fn new(name: &str, path: &str) -> Result<Self, AliasError> {
```

## Function Design

**Size:**
- Functions average 20-50 lines for domain logic
- Command functions in `src/commands/` range 15-40 lines
- Database methods stay focused on single operations (see `src/database.rs` add/remove/tag operations)

**Parameters:**
- Immutable by default: `&str`, `&Database`, `&[String]`
- Mutable when needed: `&mut Database`, `&mut Alias`
- Config passed as reference: `&Config`
- Tag operations use `&[String]` to accept both Vec and arrays

**Return Values:**
- `Result<(), Box<dyn std::error::Error>>` for command implementations
- `Result<T, SpecificError>` for internal operations
- `Option<T>` for lookups that might not exist: `db.get()` returns `Option<&Alias>`
- Empty tuple `()` used for operations with side effects only

Example from `src/database.rs` (lines 170-185):
```rust
pub fn get(&self, name: &str) -> Option<&Alias> {
    self.aliases.get(name)
}

pub fn get_mut(&mut self, name: &str) -> Option<&mut Alias> {
    self.dirty = true;
    self.aliases.get_mut(name)
}

pub fn insert(&mut self, alias: Alias) {
    self.dirty = true;
    self.aliases.insert(alias.name.clone(), alias);
}
```

## Module Design

**Exports:**
- Selective public re-exports in `lib.rs` for public API (see `src/lib.rs` lines 14-18)
- Command modules do not use `pub use` - functions exported individually
- Internal types (`DatabaseFile` in `src/database.rs` line 35) marked struct-level private by not being in public module

**Barrel Files:**
- `src/commands/mod.rs` lists all command modules but doesn't re-export (lines would show `mod register;` not `pub use`)
- Minimal re-export pattern; consumers use full paths: `commands::register::register()`

## Dirty Flag Pattern

**Optimization:**
- `Database` uses a `dirty` flag to avoid unnecessary writes (see `src/database.rs` lines 149-167)
- Flag set to `true` on all mutations: `get_mut()`, `insert()`, `remove()`, `add_tag()`
- Flag checked in `save()` to skip I/O if no changes
- Auto-saves on `Drop` via implementation of `Drop` trait (lines 366-370)

**Initialization:**
- When reading from database (line 176), `get_mut()` sets dirty=true, protecting against accidental state inconsistency

## Tag Normalization

**Pattern** (see `src/commands/register.rs` lines 67-84):
- Tags converted to lowercase
- Duplicates removed via `HashSet`
- Validated after normalization
- Always sorted: `alias.tags.sort();` (line 132 in `src/alias.rs`)

---

*Convention analysis: 2026-01-22*
