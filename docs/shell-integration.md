# Shell Integration

goto requires a shell wrapper function to change the current directory (child processes cannot change the parent shell's directory).

## Supported Shells

- **Bash** - `shell/goto.bash`
- **Zsh** - `shell/goto.zsh`
- **Fish** - `shell/goto.fish`

## How It Works

The `goto` function:
1. Calls `goto-bin` with your arguments
2. Captures the output
3. If the output is a valid directory path, runs `cd` to it
4. Otherwise, displays the output (for list, stats, help, etc.)

## fzf Integration

When [fzf](https://github.com/junegunn/fzf) is installed and you run `goto` with no arguments, an interactive picker opens:

```bash
goto                                  # Opens fzf picker
```

Features:
- Fuzzy search through all aliases
- Preview pane shows the full path
- Press Enter to navigate

### Customizing fzf

Set `GOTO_FZF_OPTS` to customize the picker:

```bash
export GOTO_FZF_OPTS="--height 80% --border rounded --preview-window right:60%"
```

Default fzf options:
```
--preview 'goto-bin -x {}'
--preview-window 'right:50%'
--height 40%
--layout reverse
--border
```

### Disabling fzf

To use list mode instead of fzf when no arguments given:

```bash
# Option 1: Uninstall fzf

# Option 2: Override in your rc file
alias goto='goto -l'   # This won't work well, instead:

# Option 3: Use list explicitly
goto -l                # Always shows list, never fzf
```

## Tab Completion

Tab completion works automatically for:
- Alias names
- Tag names (after `-t` flag)
- Command flags

The shell wrapper uses `goto-bin --names-only` and `goto-bin --tags-raw` to generate completions.

## Shell-Specific Notes

### Bash

Requires Bash 4.0+ for associative arrays. Add to `~/.bashrc`:

```bash
source ~/.config/goto/goto.bash
```

### Zsh

Works with any modern Zsh version. Add to `~/.zshrc`:

```bash
source ~/.config/goto/goto.zsh
```

### Fish

Add to `~/.config/fish/config.fish`:

```fish
source ~/.config/goto/goto.fish
```

## Troubleshooting

### "goto: command not found"

The shell wrapper isn't loaded. Check:
1. Source line exists in your rc file
2. Shell file exists at the path
3. Restart your shell or run `source ~/.config/goto/goto.[shell]`

### "goto-bin: command not found"

The binary isn't in PATH. Check:
1. Binary location: `which goto-bin`
2. Add to PATH if needed: `export PATH="$HOME/.local/bin:$PATH"`

### cd not working

If `goto` prints the path but doesn't change directory:
1. Make sure you're using the shell function, not calling `goto-bin` directly
2. Check that the shell wrapper is sourced (see above)
