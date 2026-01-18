#!/bin/bash
# goto shell wrapper for bash
# Source this file in your .bashrc: source /path/to/goto.bash

goto() {
    local output
    local exit_code

    output=$(goto-bin "$@")
    exit_code=$?

    case "$1" in
        ""|-h|--help|-v|--version|-l|--list|-c|--cleanup|-x|--expand|--list-aliases|--names-only)
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

# Bash completion
_goto_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Complete flags
    if [[ "$cur" == -* ]]; then
        COMPREPLY=($(compgen -W "--export --import --rename --stats --recent --recent-clear --tag --untag --tags --filter --sort --config -l -r -u -p -b -f -c -h -v -x -o" -- "$cur"))
        return
    fi

    case "$prev" in
        --import)
            # Complete with files
            COMPREPLY=($(compgen -f -- "$cur"))
            return
            ;;
        --filter)
            # Complete with tags
            COMPREPLY=($(compgen -W "$(goto-bin --tags-raw 2>/dev/null)" -- "$cur"))
            return
            ;;
        --sort)
            # Complete with sort options
            COMPREPLY=($(compgen -W "alpha usage recent" -- "$cur"))
            return
            ;;
        --tag|--untag)
            # After --tag/--untag, first arg is alias, second is tag
            # Count how many args after the flag
            local flag_pos=-1
            for ((i=0; i<${#COMP_WORDS[@]}; i++)); do
                if [[ "${COMP_WORDS[i]}" == "--tag" || "${COMP_WORDS[i]}" == "--untag" ]]; then
                    flag_pos=$i
                    break
                fi
            done
            local args_after_flag=$((COMP_CWORD - flag_pos))
            if [[ $args_after_flag -eq 1 ]]; then
                # First arg: alias names
                COMPREPLY=($(compgen -W "$(goto-bin --names-only 2>/dev/null)" -- "$cur"))
            elif [[ $args_after_flag -eq 2 ]]; then
                # Second arg: tag names
                COMPREPLY=($(compgen -W "$(goto-bin --tags-raw 2>/dev/null)" -- "$cur"))
            fi
            return
            ;;
        --rename)
            # After --rename, first arg is old alias, second is new name
            local flag_pos=-1
            for ((i=0; i<${#COMP_WORDS[@]}; i++)); do
                if [[ "${COMP_WORDS[i]}" == "--rename" ]]; then
                    flag_pos=$i
                    break
                fi
            done
            local args_after_flag=$((COMP_CWORD - flag_pos))
            if [[ $args_after_flag -eq 1 ]]; then
                # First arg: existing alias names
                COMPREPLY=($(compgen -W "$(goto-bin --names-only 2>/dev/null)" -- "$cur"))
            fi
            # Second arg: new name (no completion)
            return
            ;;
        -r|--register)
            # First arg is alias name (no completion), second is directory
            if [[ ${COMP_CWORD} -eq 3 ]]; then
                COMPREPLY=($(compgen -d -- "$cur"))
            fi
            return
            ;;
        -u|--unregister|-x|--expand|-p|--push)
            COMPREPLY=($(compgen -W "$(goto-bin --names-only 2>/dev/null)" -- "$cur"))
            return
            ;;
        goto)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "--export --import --rename --stats --recent --recent-clear --tag --untag --tags --filter --sort --config -l -r -u -p -x -c -o -v -h" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "$(goto-bin --names-only 2>/dev/null)" -- "$cur"))
            fi
            return
            ;;
        *)
            COMPREPLY=($(compgen -W "$(goto-bin --names-only 2>/dev/null)" -- "$cur"))
            return
            ;;
    esac
}

complete -F _goto_completions goto
