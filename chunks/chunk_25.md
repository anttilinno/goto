# Chunk 25: Recent Directories

## Objective
Track and display recently visited directories.

## Tasks

### 1. Recent Command
```
goto --recent    List recently visited directories
```

Output (combines stack + usage data):
```
Recently Visited:
  1. dev        /home/user/dev           (2 minutes ago)
  2. projects   /home/user/projects      (1 hour ago)
  3. blog       /var/www/blog            (3 hours ago)
  4. dotfiles   /home/user/.config       (1 day ago)
  5. downloads  /home/user/Downloads     (2 days ago)
```

### 2. Implementation
```go
func (c *Commands) Recent(limit int) ([]RecentEntry, error) {
    // Get all aliases with last_used timestamp
    // Sort by last_used descending
    // Return top N
}

type RecentEntry struct {
    Alias    string
    Path     string
    LastUsed time.Time
}
```

### 3. Clear Recent History
```
goto --recent-clear    Clear last_used timestamps
```

```go
func (c *Commands) ClearRecent() error {
    // Set last_used to zero for all aliases
    // Reset use_count to 0 (optional, or separate flag)
}
```

### 4. Quick Navigation to Recent
```
goto --recent 1    Navigate to most recent
goto --recent 3    Navigate to 3rd most recent
```

### 5. Integration with Stack
- Recent list includes entries from directory stack
- Stack entries that are aliases show alias name
- Non-alias stack entries shown as path only

### 6. Default Limit
- Show 10 most recent by default
- Configurable: `goto --recent 20`

## Files to Modify
- `internal/commands/commands.go` - Add Recent, ClearRecent functions
- `cmd/goto/main.go` - Add `--recent`, `--recent-clear` flags

## Verification
- [ ] `--recent` shows recently visited aliases
- [ ] Results sorted by last_used
- [ ] `--recent-clear` clears history
- [ ] `--recent N` navigates to Nth recent
- [ ] Relative time displayed correctly
