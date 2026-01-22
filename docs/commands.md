# Commands Reference

Complete reference for all goto commands.

## Navigation

### Navigate to alias

```bash
goto <alias>        # Navigate to registered alias
goto               # Interactive fzf picker (if fzf installed)
```

If the alias doesn't exist, goto suggests similar aliases using fuzzy matching.

### Expand path

```bash
goto -x <alias>     # Print path without navigating
goto --expand <alias>
```

Useful for scripting or verifying an alias path.

## Alias Management

### Register alias

```bash
goto -r <alias> [path]              # Register alias (default: current dir)
goto --register <alias> [path]
goto -r <alias> [path] -t <tag>     # Register with tag
```

**Examples:**
```bash
goto -r proj                        # Register 'proj' as current directory
goto -r work ~/projects/work        # Register 'work' with specific path
goto -r api ~/code/api -t backend   # Register with 'backend' tag
```

### Unregister alias

```bash
goto -u <alias>                     # Remove alias
goto --unregister <alias>
```

### Rename alias

```bash
goto --rename <old> <new>           # Rename alias
```

### List aliases

```bash
goto -l                             # List all aliases (table format)
goto --list
goto -l -t <tag>                    # Filter by tag
goto --names-only                   # Just names (for scripting/completion)
```

**Output columns:** Name, Path, Uses (if stats enabled), Tags (if tags enabled)

## Tags

### Add tag

```bash
goto --tag <alias> <tag>            # Add tag to alias
```

### Remove tag

```bash
goto --untag <alias> <tag>          # Remove tag from alias
```

### List tags

```bash
goto --list-tags                    # Show all tags with alias counts
goto --tags-raw                     # Just tag names (for scripting)
```

## Directory Stack

Push/pop navigation like `pushd`/`popd`.

### Push

```bash
goto -p <alias>                     # Push current dir, navigate to alias
goto --push <alias>
```

### Pop

```bash
goto -o                             # Pop and return to previous directory
goto --pop
```

## Statistics

### Usage stats

```bash
goto --stats                        # Top 10 most-used aliases
```

Shows: Rank, Name, Uses, Last Used

### Recent directories

```bash
goto --recent                       # Show recently visited aliases
goto --recent <n>                   # Navigate to nth recent (1-20)
goto --recent-clear                 # Clear recent history
```

## Data Management

### Export

```bash
goto --export <file>                # Export aliases to TOML file
goto --export aliases.toml
```

### Import

```bash
goto --import <file>                # Import aliases from TOML file
goto --import aliases.toml --merge  # Merge with existing (default)
goto --import aliases.toml --replace # Replace all aliases
goto --import aliases.toml --skip   # Skip existing aliases
```

### Cleanup

```bash
goto --cleanup                      # Remove aliases with invalid paths
goto -c
goto --cleanup --dry-run            # Preview without removing
```

## Configuration

### Show config

```bash
goto --config                       # Display current configuration
```

### Version

```bash
goto -v                             # Show version (and update status)
goto --version
```

## Self-Update

```bash
goto -U                             # Download and install latest version
goto --update
```

Checks GitHub releases, verifies checksum, and updates in place.

## Help

```bash
goto -h                             # Show help
goto --help
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Alias not found / stack empty |
| 2 | Directory no longer exists |
| 3 | Invalid alias/tag format |
| 4 | Alias already exists |
| 5 | System/IO error |
