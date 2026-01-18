# Chunk 26: Shell Integration Updates

## Objective
Update shell scripts to support new commands and provide completions.

## Tasks

### 1. Update goto.bash
```bash
# Add completions for new flags
_goto_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Complete flags
    if [[ "$cur" == -* ]]; then
        COMPREPLY=($(compgen -W "--export --import --rename --stats --recent --recent-clear --tag --untag --tags --filter --sort --config -l -r -u -p -b -f -c -h" -- "$cur"))
        return
    fi

    # Complete after --import with files
    if [[ "$prev" == "--import" ]]; then
        COMPREPLY=($(compgen -f -- "$cur"))
        return
    fi

    # Complete after --filter with tags
    if [[ "$prev" == "--filter" ]]; then
        COMPREPLY=($(compgen -W "$(goto-bin --tags-raw)" -- "$cur"))
        return
    fi

    # Complete after --sort with options
    if [[ "$prev" == "--sort" ]]; then
        COMPREPLY=($(compgen -W "alpha usage recent" -- "$cur"))
        return
    fi

    # Complete aliases
    COMPREPLY=($(compgen -W "$(goto-bin -l --names-only)" -- "$cur"))
}
```

### 2. Update goto.zsh
```zsh
# Similar updates for zsh completion
_goto() {
    local -a commands
    commands=(
        '--export:Export aliases to TOML'
        '--import:Import aliases from file'
        '--rename:Rename an alias'
        '--stats:Show usage statistics'
        '--recent:Show recently visited'
        '--recent-clear:Clear recent history'
        '--tag:Add tag to alias'
        '--untag:Remove tag from alias'
        '--tags:List all tags'
        '--filter:Filter by tag'
        '--sort:Sort list'
        '--config:Show configuration'
    )
    # ... completion logic
}
```

### 3. Update goto.fish
```fish
# Fish completions
complete -c goto -l export -d 'Export aliases to TOML'
complete -c goto -l import -d 'Import aliases from file' -r
complete -c goto -l rename -d 'Rename an alias'
complete -c goto -l stats -d 'Show usage statistics'
complete -c goto -l recent -d 'Show recently visited'
complete -c goto -l recent-clear -d 'Clear recent history'
complete -c goto -l tag -d 'Add tag to alias'
complete -c goto -l untag -d 'Remove tag from alias'
complete -c goto -l tags -d 'List all tags'
complete -c goto -l filter -d 'Filter by tag' -xa '(goto-bin --tags-raw)'
complete -c goto -l sort -d 'Sort list' -xa 'alpha usage recent'
complete -c goto -l config -d 'Show configuration'
```

### 4. Add Helper Flags
Add to goto-bin for completion support:
```
goto-bin --names-only    Output alias names only (for completion)
goto-bin --tags-raw      Output tags only (for completion)
```

### 5. Handle New Output Formats
Ensure shell wrapper correctly handles:
- Multi-line output from --stats
- TOML output from --export
- Error messages with suggestions

## Files to Modify
- `shell/goto.bash` - Bash completions
- `shell/goto.zsh` - Zsh completions
- `shell/goto.fish` - Fish completions
- `cmd/goto/main.go` - Add `--names-only`, `--tags-raw` helper flags

## Verification
- [ ] Bash completions work for new flags
- [ ] Zsh completions work for new flags
- [ ] Fish completions work for new flags
- [ ] Tab-complete tags after --filter
- [ ] Tab-complete sort options after --sort
