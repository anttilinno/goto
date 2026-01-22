# Codebase Concerns

**Analysis Date:** 2026-01-22

## Tech Debt

**Platform-specific update functionality:**
- Issue: Update mechanism (`src/commands/update.rs`) is hardcoded to Linux x86_64 only
- Files: `src/commands/update.rs` (lines 153-162)
- Impact: Binary self-updates only work on Linux x86_64. Other platforms get "No download URL available for your platform" error, making `--update` and `--check-update` commands non-functional
- Fix approach: Either add platform detection for macOS/Windows asset names, or document Linux-only requirement and fail gracefully with clear messaging

**SHA256 checksum verification depends on system binary:**
- Issue: Update verification uses `sha256sum` command via subprocess instead of a crypto library
- Files: `src/commands/update.rs` (lines 237-262)
- Impact: Requires `sha256sum` to be installed on system. Update will fail if not present. Not portable across platforms. Code comment acknowledges this (line 246)
- Fix approach: Add `sha2` crate dependency and implement native SHA256 verification

**Error detection via string matching in main.rs:**
- Issue: `handle_error()` function maps error types to exit codes by searching error message strings
- Files: `src/main.rs` (lines 216-232)
- Impact: Fragile error categorization - if error messages change, exit codes may become incorrect. Unrelated errors containing keywords like "not found" or "already exists" may map to wrong codes
- Fix approach: Implement custom error types that carry error codes instead of relying on message content

**Shell RC file modification without backup:**
- Issue: Install command appends to shell rc files (.bashrc, .zshrc, config.fish) without creating backups
- Files: `src/commands/install.rs` (lines 134-157)
- Impact: If installation is interrupted or fails during rc file write, shell might become non-functional. No rollback mechanism
- Fix approach: Create backup of rc file before modification, implement atomic writes

**Recent command argument parsing ambiguity:**
- Issue: `--recent` command with numeric argument has complex heuristics to determine if arg is navigate count or display count
- Files: `src/cli.rs` (lines 185-209)
- Impact: Logic is fragile - numbers 1-20 with exactly 3 args navigate, others display. Users might expect `goto --recent 15` to show 15 items but it navigates to 15th instead
- Fix approach: Separate commands or explicit flags like `--recent-show=15` vs `--recent-goto=15`

## Known Bugs

**Fuzzy matching triggers on incomplete matches:**
- Symptoms: Typos in alias names may navigate to unintended aliases if single match found with >= 0.7 similarity score
- Files: `src/commands/navigate.rs` (lines 43-60)
- Trigger: Any typo that produces single fuzzy match with high confidence automatically navigates without confirmation
- Workaround: Use full alias name or disambiguate with --expand before navigating

**Directory validation race condition:**
- Symptoms: Alias points to valid directory at registration, but cleanup detects it as invalid before next use
- Files: `src/commands/cleanup.rs` (lines 10-14), `src/alias.rs` (lines 109-119)
- Trigger: Directory deleted between when alias was verified and when cleanup runs
- Current mitigation: `cleanup --dry-run` allows inspection before removal
- Workaround: Use `goto --expand` to verify paths are still valid before cleanup

**Import with rename strategy infinite loop potential:**
- Symptoms: If import file contains many aliases with same name, find_unique_name() increments suffix indefinitely
- Files: `src/commands/import_export.rs` (lines 140-149)
- Trigger: Import file with 1000+ copies of same alias name using Rename strategy
- Current state: Loop will eventually find unused name, but performance degrades with each collision
- Workaround: Clean import files before importing with Rename strategy

## Security Considerations

**Self-update downloads from GitHub without build verification:**
- Risk: Downloaded binary could be compromised if GitHub account is hacked or MITTS attack occurs
- Files: `src/commands/update.rs` (lines 299-315)
- Current mitigation: SHA256 checksum verification against checksums.txt (lines 318-336). However, checksums.txt is also downloaded from GitHub (same risk)
- Recommendations:
  - Sign releases with GPG key stored separately from GitHub
  - Document verification procedure for users
  - Make binary hash pinning configurable for security-conscious users

**Shell rc file source command without quoting:**
- Risk: RC file modification constructs source line without shell-escaping the path
- Files: `src/commands/install.rs` (line 111)
- Current state: `source /path/to/goto.bash` - if path contains spaces or special chars, shell will break
- Recommendations: Quote path in generated source line: `source "$path"`

**Directory path input not validated for shell injection:**
- Risk: Paths registered as aliases are not validated; malicious paths could cause issues if used in unquoted contexts
- Files: `src/alias.rs` (lines 109-119) - only checks non-empty
- Current mitigation: Shell wrappers quote the path output when executing cd
- Recommendations: Document that paths should be treated as untrusted user input; audit shell scripts for proper quoting

**Update binary written to temporary file in same directory:**
- Risk: Temporary file `.goto-bin.new` is world-readable before being made executable
- Files: `src/commands/update.rs` (lines 299-345)
- Current mitigation: Umask respected, but not explicitly enforced
- Recommendations: Create temp file with restricted permissions (0600) before writing, or use `/tmp` with explicit mode

## Performance Bottlenecks

**Database loads entire TOML file into HashMap on every operation:**
- Problem: Every command (navigate, list, cleanup) loads entire database from disk
- Files: `src/database.rs` (lines 98-107)
- Cause: TOML parsing creates full Vec then populates HashMap - no lazy loading or caching
- Improvement path: For very large alias databases (1000+ entries), consider lazy loading or caching strategy. Currently acceptable for typical use (10-100 aliases)

**Fuzzy matching recalculates similarity for all aliases on typo:**
- Problem: Each navigation typo triggers O(n) similarity calculations for all alias names
- Files: `src/fuzzy.rs` (referenced in `src/commands/navigate.rs` lines 36, 97)
- Cause: No caching of similarity scores between operations
- Improvement path: Pre-compute similarity matrix on database load if database size > 500. Most users won't notice with typical 50-100 aliases

**Update check makes synchronous HTTP requests:**
- Problem: `notify_if_update_available()` can block on network timeout during navigation
- Files: `src/commands/update.rs` (lines 204-228), `src/main.rs` (line 209)
- Cause: Uses blocking `reqwest` client with 10 second timeout
- Improvement path: Spawn async task or thread for update checks to avoid blocking user

## Fragile Areas

**Shell wrapper dependency:**
- Files: `src/commands/install.rs` (lines 8-15, embedded scripts)
- Why fragile: Binary outputs paths to stdout; shell wrappers must capture and execute `cd`. If wrapper scripts are lost or modified, navigation breaks
- Safe modification: Test shell scripts thoroughly before changing. Maintain separate versions for bash/zsh/fish - incompatibilities are common
- Test coverage: Integration tests in `tests/integration.rs` cover binary output but not shell script execution

**Directory existence checks create TOCTOU issues:**
- Files: `src/commands/navigate.rs` (lines 16-22), `src/commands/cleanup.rs` (lines 12-13)
- Why fragile: Directory can be deleted between check and use. Cleanup can remove valid aliases if race condition occurs
- Safe modification: Document that cleanup should be run when user has exclusive access to alias directory. Consider locking mechanism if reliability required

**Fuzzy matching threshold configuration:**
- Files: `src/config.rs` (lines 24-32), `src/database.rs` (line 341)
- Why fragile: Default fuzzy_threshold 0.3 vs navigate.rs high-confidence threshold of 0.7 use different scales or may not align
- Safe modification: Test thoroughly that config threshold value is properly used in fuzzy matching. Document scale (0.0-1.0 vs 0-1000)

**Configuration parsing silently uses defaults:**
- Files: `src/config.rs` (lines 137-142)
- Why fragile: Missing or invalid config.toml silently uses defaults without warning. Users may think settings are applied when they're not
- Safe modification: Add optional warning log if config file is missing, or validate config format on load

## Scaling Limits

**Single TOML file for entire database:**
- Current capacity: Tested with 500+ aliases
- Limit: When database reaches 10,000+ entries, TOML parsing becomes slow, file becomes unwieldy
- Scaling path: Consider database split (sharded by first letter) or migrate to SQLite for very large deployments

**All operations load complete database:**
- Current capacity: Fast for <1000 aliases
- Limit: Operations like list/cleanup become O(n) for all operations
- Scaling path: Implement read-only mode with mmap'd file for large databases, or database indexing

**Shell history contains all navigated paths:**
- Current capacity: Shell history unlimited by goto
- Limit: Users with 10,000+ navigations have bloated shell history
- Scaling path: Implement history pruning strategy, cap recent history at 1000 entries

## Dependencies at Risk

**Manual argument parsing instead of clap/structopt:**
- Risk: As command set grows, manual parsing becomes error-prone and hard to maintain
- Files: `src/cli.rs` (lines 80-250)
- Impact: Adding new flags requires careful manual parsing logic. No validation framework
- Migration plan: Consider switching to `clap` v4 when ready for major refactor. Current approach acceptable for current command set

**reqwest for HTTP with blocking client:**
- Risk: Network operations can hang if GitHub is slow/unreachable
- Files: `src/commands/update.rs` (lines 110-124, 302-307)
- Impact: Update checks can cause 10+ second delays during navigation if network is poor
- Alternative: Use lightweight HTTP client like `ureq` (blocking), or implement async properly

**TOML format for config files:**
- Risk: TOML parsing errors crash the entire tool
- Files: `src/config.rs` (line 139), `src/database.rs` (line 100)
- Impact: Corrupted config.toml makes goto unusable. No recovery mechanism
- Mitigation: Config parsing uses serde defaults which helps, but database TOML corruption is fatal
- Improvement: Implement atomic writes and backup rotation for TOML files

## Missing Critical Features

**No conflict detection for shell aliases:**
- Problem: Can register alias that shadows shell builtin (e.g., `cd`, `echo`). Shell will use builtin, not goto function
- Blocks: Users may be confused when alias doesn't work
- Improvement: Add check against common shell builtins during registration

**No symlink handling:**
- Problem: If registered path is symlink and target is deleted, directory validation fails even if symlink should be updated
- Blocks: Use cases with symlink-heavy directory structures
- Improvement: Add `--follow-symlinks` flag and document symlink behavior

**No database locking mechanism:**
- Problem: Two goto processes can write database simultaneously, causing corruption
- Blocks: Safety when using goto from multiple shells/scripts
- Improvement: Implement file locking (flock on Unix) or atomic writes with verification

**No recovery mechanism for corrupted databases:**
- Problem: Corrupted aliases.toml makes entire tool unusable
- Blocks: Data durability guarantees
- Improvement: Implement backup rotation and corruption detection

## Test Coverage Gaps

**Shell wrapper script execution not tested:**
- What's not tested: Actual bash/zsh/fish script behavior. Only binary output is tested
- Files: `src/commands/install.rs`, `shell/` directory
- Risk: Shell scripts could have syntax errors, quoting issues, or incompatibilities
- Priority: High - shell integration is core feature

**Update mechanism not tested end-to-end:**
- What's not tested: Actual binary download, checksum verification, in-place replacement
- Files: `src/commands/update.rs`
- Risk: Update could silently fail or corrupt installation
- Priority: High - breaking update process is critical

**Concurrent database access:**
- What's not tested: Multiple goto processes accessing database simultaneously
- Files: `src/database.rs`
- Risk: Data corruption in multi-shell scenarios
- Priority: Medium - depends on typical usage patterns

**Cross-platform fuzzy matching:**
- What's not tested: Fuzzy matching with non-ASCII characters, unicode paths
- Files: `src/fuzzy.rs`
- Risk: Aliases with international characters may not fuzzy-match correctly
- Priority: Low - typically English-only aliases

**Installation with read-only home directory:**
- What's not tested: Install command when ~/.config is not writable
- Files: `src/commands/install.rs`
- Risk: Silent failure or confusing error message
- Priority: Medium - relevant for restricted environments

---

*Concerns audit: 2026-01-22*
