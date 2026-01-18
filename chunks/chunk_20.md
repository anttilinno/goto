# Chunk 20: Import/Export

## Objective
Allow users to backup and restore their aliases database.

## Tasks

### 1. Export Command
```
goto --export    Output TOML to stdout
```

```go
func (c *Commands) Export() error {
    // Load database
    // Marshal to TOML
    // Print to stdout
}
```

Usage:
```bash
goto --export > backup.toml
goto --export | pbcopy  # macOS
```

### 2. Import Command
```
goto --import <file>    Import from TOML file
```

```go
func (c *Commands) Import(filepath string, strategy string) error {
    // Read file
    // Parse TOML
    // Merge with existing based on strategy
    // Save database
}
```

### 3. Merge Strategies
- `skip` - Skip if alias already exists (default)
- `overwrite` - Overwrite existing aliases
- `rename` - Rename conflicting aliases (add suffix)

```
goto --import backup.toml --strategy=skip
goto --import backup.toml --strategy=overwrite
goto --import backup.toml --strategy=rename
```

### 4. Import Validation
- Validate all paths exist (warn if not)
- Validate alias names are valid
- Report count of imported/skipped/renamed

### 5. Output Format
Export produces valid TOML:
```toml
[[aliases]]
name = "dev"
path = "/home/user/dev"
tags = ["work"]
created = 2025-01-17T12:00:00Z
last_used = 2025-01-17T15:30:00Z
use_count = 42

[[aliases]]
name = "blog"
path = "/var/www/blog"
tags = ["web", "personal"]
created = 2025-01-15T10:00:00Z
last_used = 2025-01-16T09:00:00Z
use_count = 7
```

## Files to Modify
- `internal/commands/commands.go` - Add Export/Import functions
- `cmd/goto/main.go` - Add `--export`, `--import`, `--strategy` flags

## Verification
- [ ] Export produces valid TOML
- [ ] Import reads TOML correctly
- [ ] Skip strategy works
- [ ] Overwrite strategy works
- [ ] Rename strategy works
- [ ] Round-trip test: export → import produces identical data
