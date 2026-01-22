# Codebase Concerns

**Analysis Date:** 2026-01-22

## Tech Debt

**Platform-specific code in `src/commands/update.rs`:**
- Issue: Self-update mechanism hardcoded for Linux x86_64 only
- Files: `src/commands/update.rs` (lines 154-161)
- Impact: Users on other platforms (macOS, Windows, ARM, etc.) cannot use the `--update` or `--check-update` commands. This feature silently fails for non-Linux users.
- Fix approach: Add conditional compilation for macOS and Windows binaries. GitHub Actions workflow should build separate binaries for each platform. Alternatively, document platform limitations clearly.

**Checksum verification using system command:**
- Issue: `calculate_sha256()` in `src/commands/update.rs` (lines 237-262) spawns external `sha256sum` command instead of using a Rust crate
- Files: `src/commands/update.rs` (lines 237-262)
- Impact: Not portable across platforms; requires external tools to be installed. Security verification fails silently if `sha256sum` is not available. Missing error context.
- Fix approach: Add `sha2` or `sha256` crate to dependencies and implement native hash calculation. Remove dependency on system command.

**Unsafe shell integration installation:**
- Issue: Shell wrapper scripts are sourced in rc files without shell syntax validation
- Files: `src/commands/install.rs` (lines 74-84, 106-155)
- Impact: If shell wrapper scripts contain syntax errors, users' shell rc files become corrupted and shell fails to initialize. No recovery mechanism.
- Fix approach: Validate shell script syntax before installing. Add backup of original rc file. Provide recovery command.

**Inadequate error handling in `config.rs`:**
- Issue: `dirs::home_dir()` is deprecated; code still uses it in multiple places
- Files: `src/config.rs` (lines 224-226)
- Impact: Triggers compiler warnings. Deprecated API may be removed in future versions of `dirs` crate. Code will break on future dependency updates.
- Fix approach: Use `dirs::home_dir()` alternative (check Rust 1.70+ standard library options) or migrate to `home` crate.

**Large unwrap/expect density:**
- Issue: 426 instances of `.unwrap()` in source code indicate widespread panic risks
- Files: Throughout codebase
- Impact: Many error conditions are unhandled, causing silent panics. Particularly problematic in:
  - `src/commands/install.rs` (line 41): `env::var("SHELL").unwrap_or_default()` followed by `.unwrap_or("")` chains
  - `src/config.rs` (line 240): `shellexpand::env(path).unwrap_or(...)` - unclear error handling
  - Test code heavily uses `.unwrap()` but test panics don't surface properly
- Fix approach: Replace common unwrap patterns with proper error propagation. Audit critical paths (install, update, file I/O) first.

## Known Bugs

**Stack operation file corruption risk:**
- Symptoms: Directory stack becomes unusable after unclean shutdown or concurrent access
- Files: `src/stack.rs` (lines 30-97), `src/commands/stack.rs`
- Trigger: Multiple `goto -p` commands run concurrently, or process killed while writing to stack file
- Workaround: Manual deletion of `~/.config/goto/goto_stack` and `goto -c --dry-run` to verify database
- Impact: Medium - affects only directory stack feature; core navigation still works. Users rarely use stack feature.

**Import with invalid paths creates orphaned entries:**
- Symptoms: Importing TOML with non-existent paths creates aliases that immediately fail on navigation
- Files: `src/commands/import_export.rs` (lines 101-107)
- Trigger: Import file with paths pointing to deleted directories
- Current behavior: Warns but imports anyway (line 103-105)
- Impact: Low - warnings are printed but users may miss them. Cleanup command removes them.
- Fix approach: Add `--skip-invalid-paths` flag or default to skipping paths that don't exist.

**Recent history accumulates unbounded:**
- Symptoms: No limit on `use_count` field; very old aliases can show artificially high usage
- Files: `src/alias.rs` (line 13), `src/database.rs` (lines 243-252)
- Trigger: Long-running user accounts accumulate navigation history indefinitely
- Impact: Low - affects statistics display only, doesn't cause crashes. Can lead to misleading usage stats.
- Fix approach: Implement usage decay (older entries counted with less weight) or add `--recent-clear` option (already exists, partially addresses this).

## Security Considerations

**Self-update downloads executable without signature verification:**
- Risk: Man-in-the-middle attack could serve malicious binary
- Files: `src/commands/update.rs` (lines 265-385)
- Current mitigation: SHA256 checksum verification (lines 319-336), but checksum fetched from same GitHub API endpoint
- Recommendations:
  1. Sign releases with PGP key and verify signatures in addition to checksums
  2. Fetch checksums from a separate secure channel (e.g., signed releases file)
  3. Pin GitHub API endpoints to use HTTPS enforced (already done via reqwest)
  4. Add timeout protection (already in place - 120 seconds on line 304)

**File permissions not verified after download:**
- Risk: Downloaded binary could be executable by unprivileged users before installation
- Files: `src/commands/update.rs` (lines 338-345)
- Current mitigation: Binary made executable with 0o755 on Unix only (lines 340-345)
- Impact: Low on Unix (file already in user directory). No protection on Windows.
- Recommendations: Verify file permissions before making executable; add explicit checks for parent directory ownership.

**Backup binary not securely cleaned:**
- Risk: Old binary left in `.goto-bin.old` could be recovered and analyzed for vulnerabilities
- Files: `src/commands/update.rs` (lines 348-375)
- Current mitigation: Backup is removed after successful update (line 362)
- Impact: Low - only if update fails, backup remains but user is notified to intervene manually
- Recommendations: Securely overwrite backup file before deletion (add `secure_delete` crate or zeroing).

**Shell wrapper scripts embedded without integrity checks:**
- Risk: If Rust binary is compromised during distribution, shell wrappers are also compromised
- Files: `src/commands/install.rs` (lines 9-15)
- Current mitigation: Source scripts in rc files - wrapper script controls what binary runs
- Recommendations: Document that shell wrappers should be reviewed before installation; add checksums of embedded scripts.

**Database paths from environment variables without validation:**
- Risk: `$GOTO_DB` or `$HOME` could point to adversarial locations
- Files: `src/config.rs` (lines 212-227)
- Current mitigation: Paths are expanded and canonicalized where possible (line 244)
- Impact: Low - permissions enforced by filesystem, but could be confusing to users with unusual setup
- Recommendations: Add warning when `GOTO_DB` points to world-writable location.

## Performance Bottlenecks

**Fuzzy matching rescans entire alias list on every typo:**
- Problem: Each failed navigation triggers full fuzzy match scan of all aliases
- Files: `src/commands/navigate.rs` (lines 35-72), `src/fuzzy.rs`
- Cause: No caching of fuzzy match results; Levenshtein distance calculated for every alias
- Current impact: Negligible for typical users (< 100 aliases), but O(n) for each typo
- Improvement path: Pre-compute and cache fuzzy match matrix; only update on database changes. Implement early termination if confidence threshold exceeded.

**Database always loaded entirely into memory:**
- Problem: All aliases stored in HashMap; no lazy loading or pagination
- Files: `src/database.rs` (lines 43-52)
- Cause: HashMap-based in-memory storage design
- Current impact: Negligible for typical users (< 1000 aliases), but becomes problematic at scale
- Improvement path: For large databases (>10k aliases), consider:
  1. Lazy loading from TOML
  2. Indexing by first letter for faster lookups
  3. Switching to SQLite for >1k aliases

**Import operation loads entire file before validation:**
- Problem: `import()` reads entire TOML file into memory before checking validity
- Files: `src/commands/import_export.rs` (lines 55-64)
- Impact: Large import files (>100MB TOML) could cause OOM. No streaming parser.
- Improvement path: Implement streaming TOML parser or read file in chunks with validation.

**Update check blocks on network I/O:**
- Problem: `check_for_updates()` makes blocking HTTP calls without timeout handling for slow networks
- Files: `src/commands/update.rs` (lines 110-124, 302-307)
- Impact: Slow networks cause `notify_if_update_available()` (called on every navigate) to hang. Already has 10-second timeout but requests are sequential.
- Improvement path: Implement async update checks with spawn_blocking, or move to separate background thread.

## Fragile Areas

**Error string matching in `main.rs`:**
- Files: `src/main.rs` (lines 216-231)
- Why fragile: Exit codes determined by string matching on error messages (lines 220-230)
- Safe modification: Add error type enums instead of parsing strings. All commands should return structured errors.
- Test coverage: No tests verify exit code mapping; changes to error messages silently break exit codes
- Example fragility: If error message changes from "not found" to "alias not found", exit code changes from 1 to 5

**Database dirty flag optimization:**
- Files: `src/database.rs` (lines 50-51, 177-179)
- Why fragile: `get_mut()` sets dirty flag even for read-only lookups (line 177)
- Safe modification: Separate `get()` for reads and `get_mut()` for writes. Test to ensure dirty flag only set on actual mutations.
- Test coverage: `test_dirty_flag_not_set_on_read()` exists (lines 748-780) but covers only `get()`, not `get_mut()` reads
- Risk: Frequent database rewrites even when no changes made

**Shell environment variable parsing in `install.rs`:**
- Files: `src/commands/install.rs` (lines 40-52, 74-84)
- Why fragile: Shell detection uses string parsing of `$SHELL` env var path (line 42)
- Safe modification:
  1. Validate `$SHELL` exists before using
  2. Handle symbolic links (e.g., `/bin/bash` vs `/usr/bin/bash`)
  3. Fallback to detecting current shell via process inspection
- Test coverage: Only tests happy path (lines 653-703); no tests for missing `$SHELL` or symlinks
- Risk: Wrong shell type detection silently installs wrong wrapper

**Update cache deserialization without version checks:**
- Files: `src/commands/update.rs` (lines 54-67)
- Why fragile: Cache file deserialization silently falls back to defaults if malformed (line 64)
- Safe modification: Add version field to cache format; migrate old caches explicitly
- Test coverage: No tests for malformed cache files or version mismatches
- Risk: Cache corruption causes update system to forget latest version info

## Scaling Limits

**Tag storage grows with aliases:**
- Current capacity: No limit on number of tags per alias or total tags
- Limit: Scales O(n) with aliases; rendering tag lists becomes slow at >10k tags
- Scaling path: Add tag indexing, implement tag pagination in list view, cache frequently used tag counts

**TOML file growing unbounded:**
- Current capacity: Single `aliases.toml` file containing all aliases and metadata
- Limit: TOML parser loads entire file into memory; becomes slow at >50k aliases
- Scaling path: Implement sharding (separate files by first letter) or migrate to database format

**Directory stack no file limit:**
- Current capacity: Stack file grows with each push, one entry per line
- Limit: No trim on stack, grows unbounded
- Scaling path: Add `--stack-limit` config option (default 100), implement circular buffer behavior

## Dependencies at Risk

**reqwest 0.12 with blocking feature:**
- Risk: Blocking HTTP client is deprecated pattern; newer async Rust ecosystem prefers tokio
- Impact: Will make building async features difficult; performance limited on slow networks
- Migration plan: Investigate reqwest 0.12 end-of-life timeline. For update checks, consider:
  1. Spawning background thread with blocking client (current approach works)
  2. Migrating to `ureq` crate (lightweight, simpler, no async overhead)
  3. Moving update checks to completely separate process/daemon

**dirs 5.0 using deprecated home_dir:**
- Risk: `home_dir()` function is marked deprecated; may be removed in dirs 6.0
- Impact: Code will break on next major version bump
- Migration plan: Test with latest `dirs` crate version; plan migration to alternative home directory detection
- Workaround: Pin `dirs = "5.0"` in Cargo.toml until migration planned

**shellexpand 3.1 with potential security issues:**
- Risk: Shell variable expansion could be attack vector if paths come from untrusted sources
- Impact: Low (paths only from environment vars and user config), but warrants code review
- Migration plan: Audit `expand_path()` function (lines 230-245 in config.rs); document that paths should not be trusted from network sources

**thiserror 1.0 for error handling:**
- Risk: No known issues, but error handling pattern is becoming dated in Rust community
- Impact: Low - thiserror is stable and well-maintained
- Migration plan: None urgent; consider anyhow/custom enums if error flexibility needed

## Missing Critical Features

**No concurrency safety:**
- Problem: Multiple goto invocations can race on database file read/write
- Blocks: Reliable operation in parallel shells, automation scripts with parallel execution
- Why not implemented: Would require file locking (cross-platform issues) or moving to proper database

**No rollback mechanism for updates:**
- Problem: If binary update fails mid-execution, backup is left at `.goto-bin.old` with no automated recovery
- Blocks: Unattended updates, CI/CD integration
- Why not implemented: Complicated by self-modification constraints (can't relink while running)

**No alias validation before navigation:**
- Problem: Aliases pointing to symlinks are followed without warning about ultimate path
- Blocks: Safely auditing alias destinations, preventing directory confusion
- Why not implemented: Would require resolving symlinks, adding complexity to navigate command

## Test Coverage Gaps

**Update command download and installation:**
- What's not tested: Actual binary download, checksum verification, file replacement
- Files: `src/commands/update.rs` - no tests exist
- Risk: Update mechanism could be completely broken and unnoticed until users encounter it
- Priority: High - this is production-critical feature

**Shell integration installation edge cases:**
- What's not tested:
  - RC file doesn't exist initially
  - RC file is symlink
  - Config directory has unusual permissions
  - Concurrent installation attempts
- Files: `src/commands/install.rs` - no tests exist (install feature is untestable without filesystem changes)
- Risk: Installation fails silently or corrupts shell configuration
- Priority: High - affects every new user

**Import with symlink paths:**
- What's not tested: Importing aliases with symlink targets; symlink targets deleted after import
- Files: `src/commands/import_export.rs` - import tests only use temp directories
- Risk: Symlink following behavior undefined; cleanup might behave incorrectly
- Priority: Medium

**Concurrent database access:**
- What's not tested: Multiple goto invocations reading/writing database simultaneously
- Files: `src/database.rs` - all tests use single isolated database
- Risk: Data corruption, lost updates, or panics under concurrent load
- Priority: Medium (low probability for typical single-user scenarios)

**Fuzzy matching threshold edge cases:**
- What's not tested:
  - Query longer than alias (should still match)
  - Empty database (should handle gracefully)
  - Very similar aliases (threshold edge cases)
- Files: `src/commands/navigate.rs` - tests cover happy path only
- Risk: Unexpected fuzzy match behavior in edge cases
- Priority: Low

---

*Concerns audit: 2026-01-22*
