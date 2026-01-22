# Codebase Structure

**Analysis Date:** 2026-01-22

## Directory Layout

```
goto/
├── src/                     # Rust source code
│   ├── main.rs              # Binary entry point, CLI dispatcher
│   ├── lib.rs               # Library exports
│   ├── cli.rs               # Argument parsing and command enum
│   ├── alias.rs             # Alias struct and validation
│   ├── database.rs          # TOML-based persistent storage
│   ├── config.rs            # Configuration loading and paths
│   ├── fuzzy.rs             # Levenshtein distance and similarity matching
│   ├── stack.rs             # Directory stack for push/pop navigation
│   └── commands/            # Command implementations
│       ├── mod.rs           # Command module exports
│       ├── navigate.rs       # navigate, expand commands
│       ├── register.rs       # register, unregister, rename commands
│       ├── list.rs           # list command with sorting/filtering
│       ├── tags.rs           # tag, untag, list-tags commands
│       ├── stack.rs          # push, pop commands
│       ├── stats.rs          # stats, recent, recent-clear commands
│       ├── cleanup.rs        # cleanup command (validate aliases)
│       ├── import_export.rs  # import, export commands
│       ├── config.rs         # config command
│       ├── install.rs        # install command (shell integration)
│       └── update.rs         # update, check-update commands
├── shell/                   # Shell integration scripts
│   ├── goto.bash             # Bash wrapper function
│   ├── goto.zsh              # Zsh wrapper function
│   └── goto.fish             # Fish wrapper function
├── tests/                   # Integration tests
│   └── integration.rs        # End-to-end tests
├── bin/                     # Compiled binaries (generated)
├── target/                  # Cargo build output (generated)
├── Cargo.toml               # Rust project manifest
├── Cargo.lock               # Dependency lock file
├── .github/                 # GitHub configuration
│   └── workflows/           # CI/CD workflows
├── .planning/               # Planning documents (this directory)
│   └── codebase/            # Codebase analysis docs
├── CLAUDE.md                # Claude Code instructions
└── README.md                # Project documentation
```

## Directory Purposes

**src/:**
- Purpose: All Rust source code for the binary and library
- Contains: Main entry point, CLI parsing, data structures, persistence logic, all command implementations
- Key files: `main.rs` (entry), `lib.rs` (public API), `cli.rs` (argument parsing)

**src/commands/:**
- Purpose: Command handlers - each command maps to a module
- Contains: One module per command category (navigate, register, list, tags, etc.)
- Pattern: Each module exports public functions returning `Result<(), Box<dyn std::error::Error>>`
- Key files: All command files (navigate.rs is most frequently used)

**shell/:**
- Purpose: Shell function wrappers that call the binary and perform cd
- Contains: bash, zsh, fish implementations of the goto() function
- Key pattern: All capture `goto-bin` output and execute `cd` only for navigation commands

**tests/:**
- Purpose: Integration tests that verify end-to-end behavior
- Contains: Tests using tempdir for isolated testing
- Pattern: Each test sets GOTO_DB env var to temp directory, runs goto-bin, verifies output/exit codes

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry point (main() function at line 11)
- `src/lib.rs`: Library exports for command modules
- `shell/goto.bash`, `shell/goto.zsh`, `shell/goto.fish`: Shell function entry points

**Configuration:**
- `Cargo.toml`: Project manifest with dependencies
- `.github/workflows/`: CI/CD pipeline configuration

**Core Logic:**
- `src/cli.rs`: Argument parsing (lines 80-250)
- `src/database.rs`: Storage and retrieval
- `src/alias.rs`: Domain model and validation
- `src/fuzzy.rs`: Matching and suggestion logic

**Commands:**
- Navigation: `src/commands/navigate.rs`
- Registration: `src/commands/register.rs`
- List/Display: `src/commands/list.rs`
- Tagging: `src/commands/tags.rs`
- Stack ops: `src/commands/stack.rs`
- Stats: `src/commands/stats.rs`
- Data: `src/commands/import_export.rs`
- Maintenance: `src/commands/cleanup.rs`, `src/commands/update.rs`
- Setup: `src/commands/install.rs`

**Testing:**
- `tests/integration.rs`: Integration test suite

## Naming Conventions

**Files:**
- Command modules: `[command_name].rs` (navigate.rs, register.rs, list.rs)
- Snake_case for all filenames
- Public interface re-exported in `commands/mod.rs`

**Functions:**
- snake_case for all functions
- Command handlers follow pattern: `pub fn [command_name](db: &mut Database, ...) -> Result<(), Box<dyn std::error::Error>>`
- Helper functions private to module with leading underscore if not exported

**Types/Structs:**
- PascalCase (Alias, Database, Config, Alias, Command, Args, Stack)
- Error types follow pattern: `[Component]Error` (AliasError, DatabaseError, ConfigError, StackError)
- Enums for variants: Command, ImportStrategy, SortOrder, ShellType

**Variables:**
- snake_case (alias_name, db_path, config_dir)
- Single letter for iterators (c for candidate, m for match)
- Prefix _ for unused variables (_db for intentionally unused)

**Constants:**
- SCREAMING_SNAKE_CASE (VERSION)
- Used for regex patterns (VALID_ALIAS_PATTERN, VALID_TAG_PATTERN)

## Where to Add New Code

**New Command:**
1. Create `src/commands/[command_name].rs`
2. Export module in `src/commands/mod.rs`
3. Add variant to `Command` enum in `src/cli.rs` (lines 15-77)
4. Add parsing logic in `parse_args()` in `src/cli.rs` (lines 80-250)
5. Add dispatch in `main.rs::run()` match block (lines 123-213)
6. Return `Result<(), Box<dyn std::error::Error>>` from command function

**New Test:**
- Add to `tests/integration.rs`
- Use `tempdir()` for isolated filesystem testing
- Set `GOTO_DB` env var for custom database location
- Use `goto_bin()` helper function to spawn subprocess

**New Feature on Existing Command:**
- Modify relevant file in `src/commands/`
- Update `src/cli.rs` if adding new CLI flags/options
- Update `src/main.rs` dispatcher if changing dispatch logic

**Shared Utilities:**
- Small reusable functions → `src/fuzzy.rs` if matching-related, or specific module
- Cross-module concerns → Create new module in `src/` (pattern: already done with config.rs, fuzzy.rs, stack.rs)
- Validation logic → Add to `src/alias.rs` validate_* functions

**New Shell Integration:**
- Add function to all three shell files: `shell/goto.bash`, `shell/goto.zsh`, `shell/goto.fish`
- Follow pattern: Call `goto-bin` with args, check exit code, perform cd if navigation command
- Register in `install.rs::ShellType` enum if new shell type

## Special Directories

**src/commands/:**
- Purpose: Isolated command handlers
- Generated: No
- Committed: Yes
- Pattern: Each command is independent, imports Database/Config as needed

**shell/:**
- Purpose: Shell-specific code
- Generated: No
- Committed: Yes
- Pattern: Duplicated across bash, zsh, fish - keep in sync manually

**tests/:**
- Purpose: Integration test suite
- Generated: No (but uses tempdir for temp test data)
- Committed: Yes
- Pattern: Uses `goto_bin!()` macro to locate compiled binary at test time

**.planning/codebase/:**
- Purpose: Codebase analysis documents
- Generated: No
- Committed: Yes
- Pattern: One document per focus area (ARCHITECTURE.md, STRUCTURE.md, etc.)

**target/:**
- Purpose: Cargo build artifacts
- Generated: Yes (by `cargo build`)
- Committed: No (in .gitignore)

**bin/:**
- Purpose: Release binary output location
- Generated: Yes (by `mise run build`)
- Committed: No (in .gitignore)

---

*Structure analysis: 2026-01-22*
