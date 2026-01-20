# goto

A Rust implementation of [goto](https://github.com/iridakos/goto) - a shell utility for navigating to aliased directories with autocomplete support.

## Installation

```bash
# Install with mise (recommended)
mise run install

# Or specify options
mise run install -- --shell=zsh --bin-dir=~/bin --dry-run
```

This will:
1. Build the binary (if needed)
2. Copy `goto-bin` to `~/.local/bin/`
3. Copy shell wrapper to `~/.config/goto/`
4. Add source line to your shell rc file

### Manual Installation

```bash
# Build from source
cargo build --release

# Copy binary
cp target/release/goto-bin ~/.local/bin/

# Source the shell script in your rc file
# Bash: add to ~/.bashrc
source /path/to/goto/shell/goto.bash

# Zsh: add to ~/.zshrc
source /path/to/goto/shell/goto.zsh

# Fish: add to ~/.config/fish/config.fish
source /path/to/goto/shell/goto.fish
```

## Uninstallation

```bash
# Uninstall with mise
mise run uninstall

# Preview what will be removed
mise run uninstall -- --dry-run
```

Note: Your aliases in `~/.config/goto/aliases.toml` are preserved.

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
