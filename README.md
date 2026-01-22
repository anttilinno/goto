# goto

Fast directory navigation with aliases. A Rust implementation inspired by [iridakos/goto](https://github.com/iridakos/goto).

## Features

- **Alias directories** - Register shortcuts for frequently used paths
- **Fuzzy matching** - Typo suggestions when alias not found
- **Tags** - Organize aliases with tags, filter by tag
- **Directory stack** - Push/pop navigation like `pushd`/`popd`
- **fzf integration** - Interactive picker when fzf is installed
- **Statistics** - Track and view most-used aliases
- **Table output** - Clean, formatted output with configurable styles
- **Self-update** - Update from GitHub releases
- **Shell completion** - Tab completion for Bash, Zsh, and Fish

## Quick Start

```bash
# Download and install
curl -L https://github.com/anttilinno/goto/releases/latest/download/goto-linux-amd64 -o goto-bin
chmod +x goto-bin && mv goto-bin ~/.local/bin/
goto-bin --install

# Restart shell, then:
goto -r proj ~/projects/myproject   # Register alias
goto proj                           # Navigate
goto -l                             # List all aliases
goto                                # fzf picker (if installed)
```

## Usage

```bash
goto <alias>                        # Navigate to alias
goto -r <alias> [path]              # Register alias
goto -u <alias>                     # Unregister alias
goto -l                             # List aliases
goto -l -t <tag>                    # Filter by tag
goto --tag <alias> <tag>            # Add tag
goto -p <alias>                     # Push to stack and navigate
goto -o                             # Pop from stack
goto --stats                        # Usage statistics
goto --recent                       # Recent directories
goto -U                             # Self-update
```

## Documentation

- [Installation](docs/installation.md)
- [Commands Reference](docs/commands.md)
- [Configuration](docs/configuration.md)
- [Shell Integration](docs/shell-integration.md)

## License

MIT
