#!/bin/zsh
# goto shell wrapper for zsh
# Source this file in your .zshrc: source /path/to/goto.zsh

goto() {
    local output
    local exit_code

    # No arguments: interactive mode with fzf (if available)
    if [[ $# -eq 0 ]]; then
        if [[ -t 0 ]] && command -v fzf &>/dev/null; then
            local selected
            selected=$(goto-bin --names-only | fzf \
                --preview 'goto-bin -x {}' \
                --preview-window 'right:50%' \
                --height 40% \
                --layout reverse \
                --border \
                ${GOTO_FZF_OPTS:-})
            [[ -z "$selected" ]] && return 0
            output=$(goto-bin "$selected")
            exit_code=$?
            if [[ $exit_code -eq 0 && -n "$output" && -d "$output" ]]; then
                cd "$output" || return 1
            else
                [[ -n "$output" ]] && echo "$output"
                return $exit_code
            fi
        else
            # No fzf available or not interactive: show list
            goto-bin -l
        fi
        return $?
    fi

    output=$(goto-bin "$@")
    exit_code=$?

    case "$1" in
        -h|--help|-v|--version|-l|--list|-c|--cleanup|-x|--expand|--list-aliases|--names-only)
            echo "$output"
            ;;
        -r|--register|-u|--unregister)
            echo "$output"
            ;;
        --export|--stats|--tags|--tags-raw|--config)
            echo "$output"
            ;;
        --rename|--tag|--untag)
            echo "$output"
            ;;
        --recent|--recent-clear)
            # --recent can either display or navigate
            if [[ "$1" == "--recent" && -n "$2" && "$2" =~ ^[0-9]+$ && "$2" -le 20 && $# -eq 2 ]]; then
                # Navigation to Nth recent
                if [[ $exit_code -eq 0 && -n "$output" && -d "$output" ]]; then
                    cd "$output" || return 1
                else
                    [[ -n "$output" ]] && echo "$output"
                    return $exit_code
                fi
            else
                echo "$output"
            fi
            ;;
        --import)
            echo "$output"
            ;;
        -p|--push|-o|--pop|*)
            if [[ $exit_code -eq 0 && -n "$output" && -d "$output" ]]; then
                cd "$output" || return 1
            else
                [[ -n "$output" ]] && echo "$output"
                return $exit_code
            fi
            ;;
    esac
    return $exit_code
}

# Zsh completion
_goto() {
    local -a aliases
    local -a options
    local -a tags
    local -a sort_options

    options=(
        '-r[Register an alias]'
        '--register[Register an alias]'
        '-u[Unregister an alias]'
        '--unregister[Unregister an alias]'
        '-l[List all aliases]'
        '--list[List all aliases]'
        '-x[Expand an alias]'
        '--expand[Expand an alias]'
        '-c[Cleanup invalid aliases]'
        '--cleanup[Cleanup invalid aliases]'
        '-p[Push current dir and goto]'
        '--push[Push current dir and goto]'
        '-o[Pop and go to directory]'
        '--pop[Pop and go to directory]'
        '-v[Show version]'
        '--version[Show version]'
        '-h[Show help]'
        '--help[Show help]'
        '--export[Export aliases to TOML]'
        '--import[Import aliases from file]:file:_files'
        '--rename[Rename an alias]'
        '--stats[Show usage statistics]'
        '--recent[Show recently visited]'
        '--recent-clear[Clear recent history]'
        '--tag[Add tag to alias]'
        '--untag[Remove tag from alias]'
        '--tags[List all tags]'
        '--filter=[Filter by tag]:tag:->tags'
        '--sort=[Sort list]:order:(alpha usage recent)'
        '--config[Show configuration]'
    )

    sort_options=(
        'alpha:Sort alphabetically'
        'usage:Sort by usage count'
        'recent:Sort by last used'
    )

    _arguments -s $options '*:alias:->aliases'

    case "$state" in
        aliases)
            aliases=(${(f)"$(goto-bin --names-only 2>/dev/null)"})
            _describe 'alias' aliases
            ;;
        tags)
            tags=(${(f)"$(goto-bin --tags-raw 2>/dev/null)"})
            _describe 'tag' tags
            ;;
    esac
}

# Ensure completion system is loaded
if ! type compdef &>/dev/null; then
    autoload -Uz compinit && compinit
fi

compdef _goto goto
