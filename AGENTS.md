# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build --release          # Build release binary
cargo test                     # Run all tests
cargo test <test_name>         # Run a single test
mise run build                 # Build and copy to bin/goto-bin
```

## Architecture

**goto** is a shell utility for navigating to aliased directories. The Rust binary (`goto-bin`) handles all logic, while shell wrappers (`shell/*.bash|zsh|fish`) handle the actual `cd` command since child processes cannot change the parent shell's directory.

### Binary Output Protocol

The binary outputs directory paths to stdout for navigation commands. The shell wrapper captures this output and performs `cd "$output"`. Non-navigation commands (list, stats, help) output directly to the user. Exit codes map to error types: 1=not found, 2=directory missing, 3=invalid input, 4=already exists, 5=system error.

### Core Modules

- **database.rs**: TOML-based persistent storage with HashMap for fast lookups. Auto-migrates from old text format. Dirty-flag optimization only writes on changes. Auto-saves on Drop.
- **alias.rs**: `Alias` struct with name, path, tags, use_count, last_used, created_at. Validation via regex patterns.
- **config.rs**: Loads from `$GOTO_DB`, `$XDG_CONFIG_HOME/goto`, or `~/.config/goto`. User settings in `config.toml`.
- **fuzzy.rs**: Levenshtein distance for suggesting similar aliases on typos.
- **stack.rs**: Simple file-based directory stack for push/pop navigation.

### Commands (src/commands/)

Each command module exports functions that take `&mut Database` and return `Result<(), Box<dyn Error>>`. The main.rs dispatches based on CLI args with manual argument parsing (no clap).

### Data Files

All stored in config directory (`~/.config/goto/` by default):
- `aliases.toml` - alias database
- `config.toml` - user settings
- `goto_stack` - directory stack (one path per line)
