# Technology Stack

**Analysis Date:** 2026-01-22

## Languages

**Primary:**
- Rust 2021 edition - Core binary application (`src/main.rs` and all modules)

**Secondary:**
- Bash - Shell wrapper for bash integration (`shell/goto.bash`)
- Zsh - Shell wrapper for zsh integration (`shell/goto.zsh`)
- Fish - Shell wrapper for fish integration (`shell/goto.fish`)

## Runtime

**Environment:**
- Rust compiled binary (`goto-bin`)
- Target platform: Linux x86_64
- Build system: Cargo (Rust package manager)

**Package Manager:**
- Cargo (Rust)
- Lockfile: `Cargo.lock` (present)

## Frameworks

**Core:**
- No external framework; pure Rust standard library + dependencies for specific functionality

**CLI:**
- Manual argument parsing in `src/cli.rs` (no clap or similar CLI framework)

**Serialization:**
- TOML - For persistent storage of aliases and configuration (`src/database.rs`)
- JSON - For update cache serialization (`src/commands/update.rs`)

## Key Dependencies

**Critical:**
- `serde` 1.0 - Serialization/deserialization framework (used for TOML and JSON)
- `toml` 0.8 - TOML parsing and serialization for aliases database
- `chrono` 0.4 - Datetime handling for alias metadata (created_at, last_used)

**HTTP & Updates:**
- `reqwest` 0.12 - HTTP client for GitHub release checks (`src/commands/update.rs`)
  - Features: blocking, json
  - Used for: Checking latest version on GitHub API, downloading checksums

**Utilities:**
- `dirs` 5.0 - Cross-platform home directory detection in `src/config.rs`
- `regex` 1.10 - Pattern validation for alias and tag names in `src/alias.rs`
- `shellexpand` 3.1 - Shell variable expansion ($VAR) in path expansion (`src/config.rs`)
- `thiserror` 1.0 - Custom error type derivation (used throughout codebase)
- `serde_json` 1.0 - JSON handling for update cache

## Configuration

**Environment:**
- Loaded via `src/config.rs` from priority:
  1. `$GOTO_DB` environment variable
  2. `$XDG_CONFIG_HOME/goto`
  3. `~/.config/goto`

**Key configs stored:**
- `config.toml` - User settings (fuzzy threshold, default sort, display options, update preferences)
- `aliases.toml` - Alias database (name, path, tags, use_count, last_used, created_at)
- `goto_stack` - Directory stack file (one path per line)

**Build:**
- `Cargo.toml` - Package manifest
- `.mise.toml` - Task automation (build, test, release, install)

## Platform Requirements

**Development:**
- Rust toolchain (latest)
- `bats` (latest) - For integration tests in `tests/integration.rs`
- `mise` - Task runner for build/test automation

**Production:**
- Linux x86_64 system with bash/zsh/fish shell available
- `~/.config/goto/` directory for configuration storage
- Write permissions to shell RC files (`.bashrc`, `.zshrc`, `.config/fish/config.fish`)

## Build Process

**Commands:**
```bash
cargo build --release          # Build release binary to target/release/goto-bin
cargo test                     # Run all tests
mise run build                 # Build and copy to bin/goto-bin
mise run install               # Install binary and shell integration
```

**Artifact:**
- Release binary: `target/release/goto-bin`
- Installed to: `$HOME/.local/bin/goto-bin` (or custom via install options)

---

*Stack analysis: 2026-01-22*
