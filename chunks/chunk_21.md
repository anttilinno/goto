# Chunk 21: Alias Rename

## Objective
Allow renaming aliases without losing metadata.

## Tasks

### 1. Rename Command
```
goto --rename <old> <new>    Rename alias
```

### 2. Implementation
```go
func (c *Commands) Rename(oldName, newName string) error {
    // Validate old alias exists
    // Validate new name doesn't exist
    // Validate new name is valid format
    // Update alias name in database
    // Preserve all metadata (stats, tags, timestamps)
    // Save database
}
```

### 3. Validation Rules
- Old alias must exist
- New alias must not exist
- New name must be valid (alphanumeric, dash, underscore)
- New name must not be empty

### 4. Error Messages
```
Error: alias 'foo' does not exist
Error: alias 'bar' already exists
Error: invalid alias name 'my alias' (no spaces allowed)
```

### 5. Success Output
```
Renamed alias 'dev' to 'development'
```

## Files to Modify
- `internal/commands/commands.go` - Add Rename function
- `internal/database/database.go` - Add RenameAlias method
- `cmd/goto/main.go` - Add `--rename` flag

## Verification
- [ ] Rename works correctly
- [ ] Metadata is preserved after rename
- [ ] Error on non-existent source alias
- [ ] Error on existing target alias
- [ ] Error on invalid target name
