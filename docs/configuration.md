# Configuration

goto stores configuration in `~/.config/goto/config.toml` (or `$XDG_CONFIG_HOME/goto/config.toml`).

## Configuration File

A default config file is created on first run. Example:

```toml
[user.fuzzy]
threshold = 0.6                    # Similarity threshold for suggestions (0.0-1.0)

[user.display]
show_stats = false                 # Show usage count in list output
show_tags = true                   # Show tags in list output
default_sort = "name"              # Sort order: "name", "usage", "recent"
table_style = "unicode"            # Table style: "unicode", "ascii", "minimal"

[user.update]
auto_check = true                  # Check for updates periodically
check_interval_hours = 24          # Hours between update checks
```

## Options

### Fuzzy Matching

| Option | Default | Description |
|--------|---------|-------------|
| `threshold` | `0.6` | Minimum similarity score (0.0-1.0) for suggestions |

Higher values require closer matches. Lower values show more suggestions.

### Display

| Option | Default | Description |
|--------|---------|-------------|
| `show_stats` | `false` | Show "Uses" column in `goto -l` |
| `show_tags` | `true` | Show "Tags" column in `goto -l` |
| `default_sort` | `"name"` | Sort order: `name`, `usage`, `recent` |
| `table_style` | `"unicode"` | Table border style |

**Table styles:**

- `unicode` - Modern box-drawing characters (default)
  ```
  ╭──────────┬─────────────────────╮
  │ Name     │ Path                │
  ├──────────┼─────────────────────┤
  │ proj     │ ~/projects/myproj   │
  ╰──────────┴─────────────────────╯
  ```

- `ascii` - ASCII characters (for limited terminals)
  ```
  +----------+---------------------+
  | Name     | Path                |
  +----------+---------------------+
  | proj     | ~/projects/myproj   |
  +----------+---------------------+
  ```

- `minimal` - No borders
  ```
  Name       Path
  proj       ~/projects/myproj
  ```

### Updates

| Option | Default | Description |
|--------|---------|-------------|
| `auto_check` | `true` | Automatically check for updates |
| `check_interval_hours` | `24` | Hours between update checks |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `GOTO_DB` | Custom config directory path |
| `GOTO_FZF_OPTS` | Additional fzf options for interactive mode |

**Example:**

```bash
export GOTO_DB=~/my-goto-config
export GOTO_FZF_OPTS="--height 80% --border rounded"
```

## File Locations

Default locations (in `~/.config/goto/`):

| File | Purpose |
|------|---------|
| `config.toml` | User configuration |
| `aliases.toml` | Alias database |
| `goto_stack` | Directory stack |
| `update_cache.json` | Update check cache |

## Show Current Config

```bash
goto --config
```

Displays the current configuration values and file path.
