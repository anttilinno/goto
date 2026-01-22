# Technology Stack

**Analysis Date:** 2026-01-22

## Languages

**Primary:**
- Rust 2021 edition - All core application logic and binary compilation (`src/`)
- Bash/Zsh/Fish - Shell wrappers for directory navigation (`shell/goto.bash`, `shell/goto.zsh`, `shell/goto.fish`)

## Runtime

**Environment:**
- Linux, macOS, and Unix-like systems (via Rust's cross-platform standard library)

**Binary Target:**
- Native binary compilation with Rust toolchain
- Output: `goto-bin` executable compiled to `target/release/goto-bin`

**Package Manager:**
- Cargo (Rust package manager)
- Lockfile: `Cargo.lock` (version 4 format)

## Frameworks

**Core:**
- Standard Rust library (std) - I/O, file operations, collections, environment variables
- No web framework or application framework - standalone CLI utility

**CLI:**
- Manual argument parsing (no clap or similar framework) - see `src/cli.rs` and `src/main.rs`

**Testing:**
- Integration tests: `tests/integration.rs`
- Unit tests embedded in modules with `#[cfg(test)]`

**Build/Dev:**
- Cargo build system - defined in `Cargo.toml`
- mise task runner - defined in `.mise.toml` for build, test, install, release tasks

## Key Dependencies

**Critical:**

- **serde** 1.0 with derive feature - Serialization/deserialization framework for TOML and JSON
- **toml** 0.8 - TOML file parsing and generation for aliases database (`aliases.toml`)
- **serde_json** 1.0 - JSON serialization for update cache (`update_cache.json`)

**Data/Time:**

- **chrono** 0.4.43 with serde feature - DateTime handling, timezone support, used in alias metadata (`created_at`, `last_used`) and update caching

**System/Utilities:**

- **dirs** 5.0.1 - Cross-platform directory resolution (XDG standards, home directory detection)
- **regex** 1.10 - Pattern matching for alias name validation
- **shellexpand** 3.1 - Shell variable expansion (tilde expansion, environment variable substitution)
- **thiserror** 1.0 - Error type definition and Display implementations

**HTTP/Networking:**

- **reqwest** 0.12 with blocking and json features - HTTP client for GitHub API integration
  - Uses hyper, rustls, and TLS dependencies for HTTPS
  - Blocking mode used for synchronous requests in update check functionality

**Development:**

- **tempfile** 3.14 (dev-dependency) - Temporary file/directory creation for tests

## Configuration

**Environment:**

Database path resolution (priority order) in `src/config.rs`:
1. `$GOTO_DB` - Custom database path override
2. `$XDG_CONFIG_HOME/goto` - XDG Base Directory specification
3. `~/.config/goto` - Default fallback

Config files stored in database directory:
- `aliases.toml` - Alias storage (TOML format)
- `config.toml` - User settings (TOML format)
- `goto_stack` - Directory stack (plaintext, one path per line)
- `update_cache.json` - Update check cache (JSON format)

Shell-specific configuration:
- Bash: Source line in `~/.bashrc` pointing to `~/.config/goto/goto.bash`
- Zsh: Source line in `~/.zshrc` pointing to `~/.config/goto/goto.zsh`
- Fish: Source line in `~/.config/fish/config.fish` pointing to `~/.config/goto/goto.fish`

Shell environment:
- `GOTO_FZF_OPTS` - Custom fzf options for fuzzy selector (optional, in `shell/goto.bash` line 19)

**Build:**

- `Cargo.toml` - Package metadata, dependencies, binary target definition
- `Cargo.lock` - Locked dependency versions
- `.mise.toml` - Task definitions for build, test, clean, release, install

## Platform Requirements

**Development:**

- Rust 1.56+ (edition 2021 minimum)
- Cargo (comes with Rust)
- mise (optional, for task automation)
- BATS (shell testing framework, optional - see `.mise.toml` line 2)

**Production:**

- Linux, macOS, or other Unix-like system
- Bash, Zsh, or Fish shell
- fzf (optional - for interactive fuzzy selection when no arguments provided)
  - Detected at runtime in shell wrappers (`shell/goto.bash` line 11, 31)
  - Fallback to non-interactive list when unavailable

**Network Requirements:**

- Internet connection required for `goto --check-updates` command
- Contacts GitHub API: `https://api.github.com/repos/anttilinno/goto/releases/latest`
- Downloads release assets from GitHub when updating

---

*Stack analysis: 2026-01-22*
