# Chunk 27: Integration Tests for Enhancements

## Objective
Comprehensive tests for all new features.

## Tasks

### 1. Migration Tests
```go
// internal/database/database_test.go

func TestMigrateFromTextFormat(t *testing.T) {
    // Create old text format file
    // Load database (triggers migration)
    // Verify TOML file created
    // Verify data preserved
    // Verify backup created
}

func TestLoadExistingTOML(t *testing.T) {
    // Create TOML file with metadata
    // Load database
    // Verify all fields preserved
}
```

### 2. Import/Export Tests
```go
// internal/commands/commands_test.go

func TestExportImportRoundTrip(t *testing.T) {
    // Create database with aliases
    // Export to buffer
    // Clear database
    // Import from buffer
    // Verify identical data
}

func TestImportMergeStrategies(t *testing.T) {
    // Test skip strategy
    // Test overwrite strategy
    // Test rename strategy
}
```

### 3. Rename Tests
```go
func TestRenameAlias(t *testing.T) {
    // Create alias with metadata
    // Rename alias
    // Verify metadata preserved
    // Verify old name gone
}

func TestRenameErrors(t *testing.T) {
    // Test rename non-existent
    // Test rename to existing
    // Test invalid name
}
```

### 4. Stats Tests
```go
func TestUsageTracking(t *testing.T) {
    // Navigate to alias
    // Verify use_count incremented
    // Verify last_used updated
}

func TestStatsCommand(t *testing.T) {
    // Create aliases with different usage
    // Run stats
    // Verify correct output
}

func TestSortByUsage(t *testing.T) {
    // Create aliases with different counts
    // List with --sort=usage
    // Verify order
}
```

### 5. Tags Tests
```go
func TestAddRemoveTag(t *testing.T) {
    // Create alias
    // Add tag
    // Verify tag present
    // Remove tag
    // Verify tag gone
}

func TestFilterByTag(t *testing.T) {
    // Create aliases with different tags
    // Filter by tag
    // Verify correct subset returned
}

func TestTagsOnRegistration(t *testing.T) {
    // Register with --tags=a,b
    // Verify both tags saved
}
```

### 6. Fuzzy Matching Tests
```go
// internal/fuzzy/fuzzy_test.go

func TestLevenshteinDistance(t *testing.T) {
    cases := []struct{s1, s2 string; want int}{
        {"", "", 0},
        {"a", "a", 0},
        {"a", "b", 1},
        {"dev", "dve", 1},
        {"projects", "project", 1},
    }
    // Test each case
}

func TestFuzzySuggestions(t *testing.T) {
    // Create alias "development"
    // Query "dev"
    // Verify suggestion returned
}
```

### 7. Recent Tests
```go
func TestRecentList(t *testing.T) {
    // Navigate to multiple aliases
    // Get recent list
    // Verify order by last_used
}

func TestRecentClear(t *testing.T) {
    // Navigate to aliases
    // Clear recent
    // Verify last_used reset
}
```

### 8. Config Tests
```go
// internal/config/config_test.go

func TestLoadConfig(t *testing.T) {
    // Create config file
    // Load config
    // Verify values
}

func TestDefaultConfig(t *testing.T) {
    // No config file
    // Load config
    // Verify defaults used
}

func TestConfigMerge(t *testing.T) {
    // Partial config file
    // Load config
    // Verify partial + defaults
}
```

## Test File Structure
```
internal/
├── config/
│   └── config_test.go
├── database/
│   └── database_test.go
├── commands/
│   └── commands_test.go
└── fuzzy/
    └── fuzzy_test.go
```

## Verification
- [ ] All tests pass: `mise run test-all`
- [ ] Coverage for migration paths
- [ ] Coverage for all new commands
- [ ] Edge cases handled
