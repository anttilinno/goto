# External Integrations

**Analysis Date:** 2026-01-22

## APIs & External Services

**GitHub:**
- GitHub Releases API - Used for checking and downloading updates
  - Service: GitHub API
  - What it's used for: Fetching latest release information and checksums
  - SDK/Client: `reqwest` 0.12 (HTTP client with blocking and json features)
  - Config: `GITHUB_API_URL = "https://api.github.com/repos/anttilinno/goto/releases/latest"` (in `src/commands/update.rs`)
  - Features used:
    - Get latest release tag and assets
    - Download `checksums.txt` for verification
    - Platform detection (currently supports Linux x86_64 only)

## Data Storage

**Databases:**
- None - Pure local file-based storage

**File Storage:**
- Local filesystem only
  - TOML format: `~/.config/goto/aliases.toml` - Alias database
  - TOML format: `~/.config/goto/config.toml` - User configuration
  - Text format: `~/.config/goto/goto_stack` - Directory stack (one path per line)
  - JSON format: `~/.config/goto/update_cache.json` - Cached update check info

**Location configuration:**
- Via environment variables (highest priority):
  - `$GOTO_DB` - Override entire database directory
  - `$XDG_CONFIG_HOME/goto` - XDG Base Directory spec
  - Default: `~/.config/goto`

**Caching:**
- Update check cache only
  - File: `update_cache.json`
  - Contains: last_check timestamp, latest_version, download_url, checksum
  - Cache TTL: Configurable via `[update] check_interval_hours` in config.toml (default 24 hours)

## Authentication & Identity

**Auth Provider:**
- None required
- Public GitHub API used (no authentication credentials needed)

## Monitoring & Observability

**Error Tracking:**
- None - Errors output to stderr with appropriate exit codes

**Logs:**
- Stdout for navigation commands (outputs directory path)
- Stderr for non-navigation output (help, errors, stats, list)
- File-based state: timestamps in alias metadata via `chrono` crate

**Exit Codes:**
- 0 - Success
- 1 - Not found (alias or stack empty)
- 2 - Directory missing (target doesn't exist)
- 3 - Invalid input (alias/tag validation failed)
- 4 - Already exists (alias already registered)
- 5 - System error (I/O, config, network, etc.)

## CI/CD & Deployment

**Hosting:**
- Not applicable (command-line tool)
- Distributed via GitHub Releases

**CI Pipeline:**
- None detected in codebase

**Release Management:**
- Manual via `mise run release [VERSION]` task
- Creates git tag and pushes to GitHub
- Binary builds and releases managed external to codebase

## Environment Configuration

**Required env vars:**
- None (all optional)

**Optional env vars:**
- `GOTO_DB` - Custom database directory path
- `XDG_CONFIG_HOME` - XDG Base Directory specification
- `HOME` - Used by `dirs` crate for home directory detection

**Secrets location:**
- Not applicable - No secrets stored
- All data is local user configuration files

## Path Expansion

**Shell Variable Expansion:**
- `shellexpand` 3.1 crate in `src/config.rs::expand_path()`
- Expands `$VAR` and `${VAR}` environment variables in alias paths
- Handles tilde expansion (`~`) manually before using `shellexpand`

## Webhooks & Callbacks

**Incoming:**
- None - CLI tool only

**Outgoing:**
- None - No external callbacks

## Shell Integration

**Shell Wrappers:**
- `shell/goto.bash` - Bash integration sourced in `.bashrc`
- `shell/goto.zsh` - Zsh integration sourced in `.zshrc`
- `shell/goto.fish` - Fish integration sourced in `.config/fish/config.fish`

**Shell Integration Protocol:**
- Binary outputs directory path to stdout
- Shell wrapper captures output and executes `cd "$output"`
- Non-navigation commands (list, stats, help) output directly to user via stderr

---

*Integration audit: 2026-01-22*
