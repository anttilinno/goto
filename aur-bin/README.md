# goto-bin AUR Package

Pre-built binary package for [goto](https://github.com/anttilinno/goto) - a shell utility for navigating to aliased directories.

## Installation

```bash
# Using yay
yay -S goto-bin

# Using paru
paru -S goto-bin

# Manual
git clone https://aur.archlinux.org/goto-bin.git
cd goto-bin
makepkg -si
```

## Post-Installation

Add to your shell configuration:

**Bash** (`~/.bashrc`):
```bash
source /usr/share/goto/goto.bash
```

**Zsh** (`~/.zshrc`):
```bash
source /usr/share/goto/goto.zsh
```

**Fish** (`~/.config/fish/config.fish`):
```fish
source /usr/share/goto/goto.fish
```

## Maintenance

To update `.SRCINFO` after modifying `PKGBUILD`:
```bash
makepkg --printsrcinfo > .SRCINFO
```

To update checksums:
```bash
updpkgsums
```
