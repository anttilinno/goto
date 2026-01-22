# External Integrations

**Analysis Date:** 2026-01-22

## APIs & External Services

**GitHub Releases API:**
- Service: GitHub releases API for self-update checking and downloading
- What it's used for: Check for new versions, download updated binary, verify checksums
- SDK/Client: `reqwest` blocking HTTP client (version 0.12)
- Endpoint: `https://api.github.com/repos/anttilinno/goto/releases/latest`
- Auth: User-Agent header with version (`goto/{version}`)
- Timeout: 10 seconds per request
- Location: `src/commands/update.rs` lines 110-141

**GitHub Release Assets:**
- Download URL: Dynamically obtained from release JSON
- Checksum file: `checksums.txt` from release assets
- Binary naming pattern: `goto-linux-amd64` (platform-specific)
- Location: `src/commands/update.rs` lines 127-149

## Data Storage

**Local File Storage:**

**Aliases Database:**
- Type: TOML format
- Path: `~/.config/goto/aliases.toml` (or `$GOTO_DB/aliases.toml`)
- Format: Array of Alias objects with fields: name, path, tags, use_count, last_used, created_at
- Persistence: HashMap in-memory with dirty-flag optimization (writes only on changes)
- Location: `src/database.rs` (Database struct)
- Auto-migration: From legacy plaintext format to TOML on first load

**Configuration:**
- Type: TOML format
- Path: `~/.config/goto/config.toml`
- Contents: User settings for general, display, and update behavior
- Location: `src/config.rs` (UserConfig, GeneralConfig, DisplayConfig, UpdateConfig structs)

**Directory Stack:**
- Type: Plaintext
- Path: `~/.config/goto/goto_stack`
- Format: One directory path per line
- Location: `src/stack.rs`

**Update Cache:**
- Type: JSON format
- Path: `~/.config/goto/update_cache.json`
- Contents: Caching of latest version check results
- Fields: last_check (DateTime), latest_version, download_url, checksum
- Location: `src/commands/update.rs` lines 15-33

**File Storage:**

No external file storage services (S3, etc.). All data stored locally in user's config directory.

**Caching:**

- In-process HashMap caching for alias lookups (DatabaseError struct)
- File-based caching for update checks with timestamp to avoid excessive GitHub API calls
- Update check cache interval: 24 hours by default (configurable via `update.check_interval_hours`)

## Authentication & Identity

**Auth Provider:**

- None - No user authentication required
- Command-line based access control (binary runs with user's shell permissions)

**Shell Integration:**

- Shell wrappers (`goto.bash`, `goto.zsh`, `goto.fish`) provide user-facing CLI
- Shell wrappers detect and use fzf for interactive selection if available
- No authentication tokens or API keys required for basic usage

## Monitoring & Observability

**Error Tracking:**

- None detected - No integration with Sentry, Datadog, or similar

**Logs:**

- No persistent logging framework
- Output to stdout/stderr based on command type
- Standard error messages via `eprintln!()` and command output via `println!()`
- Exit codes map to error types:
  - 0: Success
  - 1: Alias not found
  - 2: Directory missing/invalid
  - 3: Invalid input
  - 4: Alias already exists
  - 5: System error
  - Location: `src/main.rs` handle_error function

**Update Check Notifications:**

- Checks for new versions automatically on startup (if enabled)
- Location: `src/commands/update.rs` lines 172-200 (run_update_check function)
- Notifies user of availability when new version detected

## CI/CD & Deployment

**Hosting:**

- GitHub (source code)
- Self-hosted releases available on GitHub Releases

**CI Pipeline:**

- Not detected in codebase (no GitHub Actions, GitLab CI, etc. configuration files)

**Release Process:**

- Manual versioning and tagging via `mise run release [VERSION]`
- Git-based: Creates commits and tags, pushes to GitHub
- Location: `.mise.toml` lines 17-55 (release task)
- Publishes binary and checksums to GitHub Releases

## Environment Configuration

**Required Environment Variables:**

- None strictly required - all have sensible defaults

**Optional Environment Variables:**

- `GOTO_DB` - Override default database path (default: `$XDG_CONFIG_HOME/goto` or `~/.config/goto`)
- `XDG_CONFIG_HOME` - XDG Base Directory standard (default: `~/.config` if not set)
- `HOME` - User home directory (standard system variable)
- `SHELL` - Current shell path (detected for installation)
- `GOTO_FZF_OPTS` - Custom options to pass to fzf during interactive selection
  - Location: `shell/goto.bash` line 19

**Secrets Location:**

- No secrets are managed by goto - it's a local utility
- No API keys or credentials required
- GitHub API is unauthenticated (public endpoint, rate-limited)

## Webhooks & Callbacks

**Incoming:**

- None

**Outgoing:**

- None - No callbacks or webhooks to external services

## Integration Points Summary

| Integration | Type | Required | Used For |
|------------|------|----------|----------|
| GitHub Releases API | External API | Optional | Update checking, binary distribution |
| fzf | External CLI | Optional | Interactive alias selection |
| XDG Base Directory | System standard | Optional | Config directory location |
| Local filesystem | Local storage | Required | Alias database, config, stack |

---

*Integration audit: 2026-01-22*
