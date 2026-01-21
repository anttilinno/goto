# goto shell wrapper for fish
# Save to ~/.config/fish/functions/goto.fish or source in config.fish

function goto
    # No arguments: interactive mode with fzf (if available)
    if test (count $argv) -eq 0
        if isatty stdin; and type -q fzf
            set -l selected (goto-bin --names-only | fzf \
                --preview 'goto-bin -x {}' \
                --preview-window 'right:50%' \
                --height '40%' \
                --layout reverse \
                --border \
                $GOTO_FZF_OPTS)
            test -z "$selected"; and return 0
            set -l output (goto-bin $selected)
            set -l exit_code $status
            if test $exit_code -eq 0 -a -n "$output" -a -d "$output"
                cd $output
            else
                test -n "$output"; and echo $output
                return $exit_code
            end
        else
            # No fzf available or not interactive: show list
            goto-bin -l
        end
        return $status
    end

    set -l output (goto-bin $argv)
    set -l exit_code $status

    switch "$argv[1]"
        case -h --help -v --version -l --list -c --cleanup -x --expand --list-aliases --names-only -r --register -u --unregister --export --stats --tags --tags-raw --config --rename --tag --untag --import
            echo $output
        case --recent --recent-clear
            # --recent can either display or navigate
            if test "$argv[1]" = "--recent" -a (count $argv) -eq 2 -a "$argv[2]" -le 20 2>/dev/null
                # Navigation to Nth recent
                if test $exit_code -eq 0 -a -n "$output" -a -d "$output"
                    cd $output
                else
                    test -n "$output" && echo $output
                    return $exit_code
                end
            else
                echo $output
            end
        case '*'
            if test $exit_code -eq 0 -a -n "$output" -a -d "$output"
                cd $output
            else
                test -n "$output" && echo $output
                return $exit_code
            end
    end
    return $exit_code
end

# Fish completions
complete -c goto -f

# Default: complete with alias names when no flag
complete -c goto -n "not __fish_seen_subcommand_from -r --register -u --unregister -l --list -x --expand -c --cleanup -p --push -o --pop -v --version -h --help --export --import --rename --stats --recent --recent-clear --tag --untag --tags --filter --sort --config" -a "(goto-bin --names-only 2>/dev/null)"

# Basic options
complete -c goto -s r -l register -d "Register alias" -r -F
complete -c goto -s u -l unregister -d "Unregister alias" -ra "(goto-bin --names-only 2>/dev/null)"
complete -c goto -s l -l list -d "List aliases"
complete -c goto -s x -l expand -d "Expand alias" -ra "(goto-bin --names-only 2>/dev/null)"
complete -c goto -s c -l cleanup -d "Cleanup invalid aliases"
complete -c goto -s p -l push -d "Push and goto" -ra "(goto-bin --names-only 2>/dev/null)"
complete -c goto -s o -l pop -d "Pop directory"
complete -c goto -s v -l version -d "Show version"
complete -c goto -s h -l help -d "Show help"

# Export/Import
complete -c goto -l export -d "Export aliases to TOML"
complete -c goto -l import -d "Import aliases from file" -r

# Rename
complete -c goto -l rename -d "Rename an alias" -ra "(goto-bin --names-only 2>/dev/null)"

# Statistics and recent
complete -c goto -l stats -d "Show usage statistics"
complete -c goto -l recent -d "Show recently visited"
complete -c goto -l recent-clear -d "Clear recent history"

# Tags
complete -c goto -l tag -d "Add tag to alias" -ra "(goto-bin --names-only 2>/dev/null)"
complete -c goto -l untag -d "Remove tag from alias" -ra "(goto-bin --names-only 2>/dev/null)"
complete -c goto -l tags -d "List all tags"

# Filtering and sorting (used with --list)
# Note: These use --filter=<tag> and --sort=<order> format
complete -c goto -l filter= -d "Filter by tag" -xa "(goto-bin --tags-raw 2>/dev/null)"
complete -c goto -l sort= -d "Sort list" -xa "alpha usage recent"

# Config
complete -c goto -l config -d "Show configuration"
