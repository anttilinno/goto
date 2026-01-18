# goto

A Rust implementation of [goto](https://github.com/iridakos/goto) - a shell utility for navigating to aliased directories with autocomplete support.

## Installation

```bash
# Build from source
cargo build --release

# Or use mise
mise run build
```

The binary will be placed in `bin/goto-bin`.

## Usage

Source the appropriate shell script for your shell:

- Bash: `source shell/goto.bash`
- Zsh: `source shell/goto.zsh`
- Fish: `source shell/goto.fish`

### Commands

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
