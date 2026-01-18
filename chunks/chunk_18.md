# Chunk 18: Database Migration to TOML

## Objective
Migrate the database from plain text format to TOML to support metadata (usage stats, tags, timestamps).

## Tasks

### 1. Add TOML Dependency
```bash
go get github.com/BurntSushi/toml
```

### 2. Create New AliasEntry Struct
```go
// internal/database/database.go

type AliasEntry struct {
    Name     string    `toml:"name"`
    Path     string    `toml:"path"`
    Tags     []string  `toml:"tags,omitempty"`
    Created  time.Time `toml:"created"`
    LastUsed time.Time `toml:"last_used,omitempty"`
    UseCount int       `toml:"use_count"`
}

type AliasDatabase struct {
    Aliases []AliasEntry `toml:"aliases"`
}
```

### 3. Update Database Interface
- Modify `Database` struct to hold `[]AliasEntry` instead of `map[string]string`
- Add helper methods to convert between formats for backward compatibility
- Implement TOML marshal/unmarshal

### 4. Implement Auto-Migration
```go
func (d *Database) migrateFromTextFormat(oldPath string) error {
    // Read old text format
    // Parse line by line: "alias path"
    // Convert to AliasEntry with default metadata
    // Save as TOML
    // Backup old file as aliases.txt.bak
}
```

### 5. File Locations
- Old: `~/.config/goto/aliases` (text)
- New: `~/.config/goto/aliases.toml`
- Backup: `~/.config/goto/aliases.txt.bak`

## New TOML Format
```toml
[[aliases]]
name = "dev"
path = "/home/user/dev"
tags = ["work"]
created = 2025-01-17T12:00:00Z
last_used = 2025-01-17T15:30:00Z
use_count = 42
```

## Migration Logic
1. On Load():
   - Check if `aliases.toml` exists → load it
   - Else check if `aliases` (text) exists → migrate it
   - Else create empty database

## Verification
- [ ] Old text database auto-converts to TOML
- [ ] All existing tests still pass
- [ ] New TOML file is created correctly
- [ ] Backup of old file is created
