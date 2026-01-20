# goto

A Rust implementation of [goto](https://github.com/iridakos/goto) - a shell utility for navigating to aliased directories with autocomplete support.

## Installation

1. Copy `goto-bin` to a directory in your PATH (e.g., `~/.local/bin/`)
2. Run `goto-bin --install` to set up shell integration

```bash
# Download or build the binary, then:
goto-bin --install                    # auto-detect shell
goto-bin --install --shell=zsh        # specify shell
goto-bin --install --skip-rc          # don't modify rc file
goto-bin --install --dry-run          # preview changes
```

This will:
1. Copy shell wrapper to `~/.config/goto/`
2. Add source line to your shell rc file (unless `--skip-rc`)

### Development Install (with mise)

```bash
mise run install    # builds binary, copies to ~/.local/bin, sets up shell
```

## Commands

```bash
# Register an alias
goto -r myproject /path/to/project

# Navigate to an alias
goto myproject

# List all aliases
goto -l

# Unregister an alias
goto -u myproject

# Rename an alias
goto --rename oldname newname

# Push directory to stack
goto -p myproject

# Pop directory from stack
goto -o

# Tag an alias
goto --tag myproject work

# Filter by tag
goto -l -t work

# Show statistics
goto --stats

# Export aliases
goto --export aliases.toml

# Import aliases
goto --import aliases.toml
```

## Development

```bash
# Run tests
cargo test

# Build release
cargo build --release
```
