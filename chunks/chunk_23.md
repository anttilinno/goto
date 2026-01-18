# Chunk 23: Tags/Groups

## Objective
Allow organizing aliases into categories using tags.

## Tasks

### 1. Add Tags on Registration
```
goto -r myproject /path/to/project --tags=work,golang
```

### 2. Tag Management Commands
```
goto --tag <alias> <tag>      Add tag to alias
goto --untag <alias> <tag>    Remove tag from alias
```

```go
func (c *Commands) AddTag(alias, tag string) error {
    // Get alias entry
    // Add tag if not already present
    // Save database
}

func (c *Commands) RemoveTag(alias, tag string) error {
    // Get alias entry
    // Remove tag if present
    // Save database
}
```

### 3. Filter by Tag
```
goto -l --filter=work    Show only aliases with 'work' tag
goto -l --filter=golang  Show only aliases with 'golang' tag
```

### 4. Display Tags in List
When `show_tags = true` in config (default):
```
goto -l
dev        /home/user/dev           [work, golang]
blog       /var/www/blog            [personal, web]
dotfiles   /home/user/.config       []
```

### 5. List All Tags
```
goto --tags    List all unique tags with counts
```

Output:
```
Tags:
  work     (5 aliases)
  personal (3 aliases)
  golang   (2 aliases)
  web      (2 aliases)
```

### 6. Tag Validation
- Tags are case-insensitive (stored lowercase)
- Tags are alphanumeric with dash/underscore
- No spaces in tags

## Files to Modify
- `internal/commands/commands.go` - Add tag commands
- `internal/database/database.go` - Tag manipulation methods
- `cmd/goto/main.go` - Add `--tag`, `--untag`, `--tags`, `--filter` flags

## Verification
- [ ] Tags can be added during registration
- [ ] `--tag` adds tag to alias
- [ ] `--untag` removes tag from alias
- [ ] `--filter` filters list by tag
- [ ] `--tags` lists all tags
- [ ] Tags are displayed in list output
