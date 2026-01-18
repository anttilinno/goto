# Chunk 22: Usage Stats

## Objective
Track and display alias usage statistics.

## Tasks

### 1. Track Usage on Navigate
Update `Navigate` command to:
- Increment `use_count`
- Update `last_used` timestamp

```go
func (c *Commands) Navigate(alias string) (string, error) {
    // Get alias entry
    entry.UseCount++
    entry.LastUsed = time.Now()
    // Save database
    // Return path
}
```

### 2. Stats Command
```
goto --stats    Show usage statistics
```

Output:
```
Usage Statistics
================
Most Used:
  1. dev        (142 uses, last: 2 hours ago)
  2. projects   (89 uses, last: 1 day ago)
  3. blog       (45 uses, last: 3 days ago)

Total aliases: 15
Total navigations: 423
```

### 3. Sort Options for List
```
goto -l --sort=usage     Sort by use count (descending)
goto -l --sort=recent    Sort by last used (most recent first)
goto -l --sort=alpha     Sort alphabetically (default)
```

### 4. Display Stats in List
When `show_stats = true` in config:
```
goto -l
dev        /home/user/dev           [142 uses]
projects   /home/user/projects      [89 uses]
blog       /var/www/blog            [45 uses]
```

## Files to Modify
- `internal/database/database.go` - Update UseCount/LastUsed on access
- `internal/commands/commands.go` - Add Stats, update List with sorting
- `cmd/goto/main.go` - Add `--stats`, `--sort` flags

## Verification
- [ ] Navigate increments use_count
- [ ] Navigate updates last_used
- [ ] `--stats` shows correct statistics
- [ ] `--sort=usage` sorts by usage
- [ ] `--sort=recent` sorts by last used
- [ ] `--sort=alpha` sorts alphabetically
