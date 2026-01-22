# Installation

## Quick Install

1. Download the latest release from [GitHub Releases](https://github.com/anttilinno/goto/releases)
2. Make it executable and move to PATH:

```bash
chmod +x goto-linux-amd64
mv goto-linux-amd64 ~/.local/bin/goto-bin
```

3. Run the installer:

```bash
goto-bin --install
```

4. Restart your shell or source the rc file.

## Install Options

```bash
goto-bin --install                    # Auto-detect shell
goto-bin --install --shell=bash       # Specify shell (bash/zsh/fish)
goto-bin --install --skip-rc          # Don't modify rc file
goto-bin --install --dry-run          # Preview changes only
```

The installer:
1. Copies the shell wrapper to `~/.config/goto/`
2. Adds a source line to your shell rc file (`.bashrc`, `.zshrc`, or `config.fish`)

## Manual Installation

If you prefer manual setup:

1. Place `goto-bin` somewhere in your PATH
2. Copy the appropriate shell file from the repo:
   - `shell/goto.bash` for Bash
   - `shell/goto.zsh` for Zsh
   - `shell/goto.fish` for Fish

3. Source it in your shell rc file:

```bash
# Bash (~/.bashrc)
source ~/.config/goto/goto.bash

# Zsh (~/.zshrc)
source ~/.config/goto/goto.zsh

# Fish (~/.config/fish/config.fish)
source ~/.config/goto/goto.fish
```

## Building from Source

Requirements: Rust 1.70+

```bash
git clone https://github.com/anttilinno/goto.git
cd goto
cargo build --release
cp target/release/goto-bin ~/.local/bin/
goto-bin --install
```

## Updating

```bash
goto -U                               # Self-update from GitHub
goto --update
```

Or download a new release and replace the binary.

## Uninstalling

1. Remove the source line from your shell rc file
2. Delete the binary: `rm ~/.local/bin/goto-bin`
3. Delete config directory: `rm -rf ~/.config/goto/`
