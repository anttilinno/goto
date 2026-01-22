# Architecture

**Analysis Date:** 2026-01-22

## Pattern Overview

**Overall:** Layered CLI application with persistent storage

**Key Characteristics:**
- Command-driven architecture where each command maps to a module
- In-memory HashMap caching over TOML file storage with dirty-flag optimization
- Separation between binary logic (Rust) and shell integration (wrapper functions)
- Error handling via exit codes (1-5) mapped to specific error types
- Fuzzy matching for typo tolerance and suggestion generation

## Layers

**CLI / Entry Point:**
- Purpose: Parse arguments and dispatch to appropriate command
- Location: `src/main.rs`, `src/cli.rs`
- Contains: Argument parsing, command routing, error handling
- Depends on: All command modules, Config, Database
- Used by: Shell wrappers (bash/zsh/fish)

**Command Layer:**
- Purpose: Implement specific user-facing operations
- Location: `src/commands/*.rs` (navigate, register, list, tags, stack, stats, import_export, cleanup, config, update, install)
- Contains: Business logic for each command type
- Depends on: Database, Config, Alias, Stack
- Used by: main.rs dispatcher

**Data Layer:**
- Purpose: Persistent storage and retrieval of aliases
- Location: `src/database.rs`
- Contains: TOML file I/O, HashMap caching, dirty-flag optimization, auto-migration from old text format
- Depends on: Alias, Config, Fuzzy
- Used by: Command modules, navigate command for lookup

**Domain Model:**
- Purpose: Core data structures and validation
- Location: `src/alias.rs`
- Contains: Alias struct (name, path, tags, use_count, last_used, created_at), validation regex patterns
- Depends on: chrono, serde
- Used by: Database, all commands

**Support Modules:**
- Purpose: Cross-cutting utilities
- Location: `src/config.rs` (configuration), `src/fuzzy.rs` (matching), `src/stack.rs` (directory stack)
- Contains: Config loading, Levenshtein distance, similarity scoring, directory stack persistence
- Depends on: Standard library, chrono, serde, thiserror
- Used by: Multiple modules as needed

**Shell Integration:**
- Purpose: Provide shell functions that use the binary
- Location: `shell/goto.bash`, `shell/goto.zsh`, `shell/goto.fish`
- Contains: `goto()` wrapper function that captures binary output and performs `cd` command
- Depends on: `goto-bin` binary and optional `fzf`
- Used by: Shell users sourcing the files

## Data Flow

**Navigation Flow:**

1. User runs `goto myalias` (shell wrapper)
2. Shell wrapper calls `goto-bin myalias`
3. `main.rs` parses args → Command::Navigate dispatched
4. `navigate::navigate()` looks up alias in Database
5. If not found, `fuzzy::find_matches()` generates suggestions with similarity scores
6. If exact match found and directory exists, `db.record_usage()` increments use_count, updates last_used timestamp
7. Path printed to stdout
8. Shell wrapper captures stdout and executes `cd "$output"`

**Registration Flow:**

1. User runs `goto -r myalias /path/to/dir`
2. `main.rs` → Command::Register dispatched
3. `register::register_with_tags()` validates:
   - Alias name against regex pattern
   - Tags (case-insensitive, alphanumeric with dash/underscore)
   - Directory exists and is readable
4. Creates Alias struct with created_at timestamp
5. `db.add_with_tags()` inserts into HashMap
6. `db.save()` only writes if dirty flag is set (optimization)
7. Confirmation message printed to stdout

**Database Persistence:**

1. On load: Check TOML file first, fall back to old text format (migration), or start empty
2. On add/modify/delete: Set dirty=true
3. On save: Only writes file if dirty=true, then sets dirty=false
4. Auto-save on Drop: Database implements Drop trait to save on exit

**State Management:**

- In-memory: HashMap<String, Alias> for O(1) lookup
- Persistent: TOML serialization to `~/.config/goto/aliases.toml` (or custom $GOTO_DB)
- Directory stack: Simple text file (`goto_stack`), one path per line
- Stats: Stored in Alias struct fields (use_count, last_used)

## Key Abstractions

**Alias:**
- Purpose: Represents a single directory shortcut with metadata
- Examples: `src/alias.rs` - Alias struct definition
- Pattern: Simple struct with serde serialization, validation functions validate_alias(), validate_tag()

**Database:**
- Purpose: Manages in-memory cache backed by TOML storage
- Examples: `src/database.rs` - load(), save(), get(), add_with_tags()
- Pattern: Lazy loading with dirty-flag optimization, auto-migration support

**CommandResult:**
- Purpose: Consistent error handling across commands
- Examples: All commands return `Result<(), Box<dyn std::error::Error>>`
- Pattern: Dynamic error boxing for uniform error propagation

**Fuzzy Matcher:**
- Purpose: Suggest corrections for typos and partial matches
- Examples: `src/fuzzy.rs` - find_matches(), similarity(), levenshtein_distance()
- Pattern: Similarity scoring with substring boost, configurable threshold

## Entry Points

**Binary Entry:**
- Location: `src/main.rs::main()`
- Triggers: User executes `goto-bin [args]`
- Responsibilities: Parse CLI args, load config, load database, dispatch to command handler, return exit code

**Command Dispatch:**
- Location: `src/main.rs::run()` lines 31-213
- Triggers: After argument parsing
- Responsibilities: Route to appropriate command module, handle pre-command-specific error codes

**Shell Wrapper:**
- Location: `shell/goto.bash`, `shell/goto.zsh`, `shell/goto.fish`
- Triggers: User executes `goto` in shell
- Responsibilities: Call goto-bin, capture output, perform cd or display output

## Error Handling

**Strategy:** Exit code mapping to semantic error types

**Patterns:**

- Exit code 0: Success
- Exit code 1: Alias not found / stack is empty
- Exit code 2: Directory no longer exists
- Exit code 3: Invalid alias/tag format
- Exit code 4: Alias already exists
- Exit code 5: System/IO error

Exit codes determined in `main.rs::handle_error()` by matching error message strings (lines 216-232).

Error types defined as enums using `thiserror` macro:
- `AliasError` in `alias.rs` (ValidationError, NotFound, AlreadyExists, DirectoryNotFound)
- `DatabaseError` in `database.rs` (IO, TOML serialization, config errors)
- `ConfigError` in `config.rs` (NoHomeDir, IO, TOML parse errors)
- `StackError` in `stack.rs` (Empty)

## Cross-Cutting Concerns

**Logging:** Uses `eprintln!()` macro directly. No structured logging framework.

**Validation:**
- Alias names: Regex pattern `^[a-zA-Z0-9][a-zA-Z0-9_.-]*$` in `alias.rs::validate_alias()`
- Tags: Regex pattern `^[a-zA-Z0-9][a-zA-Z0-9_-]*$` in `alias.rs::validate_tag()`
- Paths: Checked via filesystem existence check before registration

**Configuration:**
- Loaded from `$GOTO_DB`, `$XDG_CONFIG_HOME/goto`, or `~/.config/goto`
- User settings in `config.toml` (fuzzy_threshold, default_sort, display options, update settings)
- Settings are loaded once at startup in `main.rs::run()`

**Authentication:** None - local filesystem access only

**Concurrency:** Single-threaded, sequential command execution. Database lock not implemented.

---

*Architecture analysis: 2026-01-22*
