# Chunk 19: Config File Support

## Objective
Add user configuration file support with TOML format.

## Tasks

### 1. Create Config Struct
```go
// internal/config/config.go

type GeneralConfig struct {
    FuzzyThreshold float64 `toml:"fuzzy_threshold"`
    DefaultSort    string  `toml:"default_sort"` // alpha, usage, recent
}

type DisplayConfig struct {
    ShowStats bool `toml:"show_stats"`
    ShowTags  bool `toml:"show_tags"`
}

type Config struct {
    General GeneralConfig `toml:"general"`
    Display DisplayConfig `toml:"display"`
}
```

### 2. Default Configuration
```go
func DefaultConfig() *Config {
    return &Config{
        General: GeneralConfig{
            FuzzyThreshold: 0.6,
            DefaultSort:    "alpha",
        },
        Display: DisplayConfig{
            ShowStats: false,
            ShowTags:  true,
        },
    }
}
```

### 3. Config File Location
- Path: `~/.config/goto/config.toml`
- Create with defaults if doesn't exist

### 4. Config Loading
```go
func LoadConfig() (*Config, error) {
    // Load from ~/.config/goto/config.toml
    // Merge with defaults for missing values
    // Return config
}
```

### 5. Add CLI Flag
```
goto --config    Show current configuration
```

### 6. Config File Format
```toml
[general]
fuzzy_threshold = 0.6
default_sort = "alpha"  # alpha, usage, recent

[display]
show_stats = false
show_tags = true
```

## Files to Modify
- `internal/config/config.go` - Add TOML parsing, new settings
- `cmd/goto/main.go` - Add `--config` flag

## Verification
- [ ] Config file is created on first run
- [ ] Settings are loaded correctly
- [ ] `goto --config` displays current settings
- [ ] Missing config values use defaults
