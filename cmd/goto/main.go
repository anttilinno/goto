package main

import (
	"errors"
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/antti/goto-go/internal/alias"
	"github.com/antti/goto-go/internal/commands"
	"github.com/antti/goto-go/internal/stack"
)

var version = "1.0.0"

func main() {
	os.Exit(run())
}

func run() int {
	if len(os.Args) < 2 {
		printUsage()
		return 1
	}

	arg := os.Args[1]

	switch arg {
	case "-h", "--help":
		printHelp()
		return 0

	case "-v", "--version":
		fmt.Printf("goto-go version %s\n", version)
		return 0

	case "--config":
		if err := commands.ShowConfig(); err != nil {
			return handleError(err)
		}
		return 0

	case "-l", "--list":
		// Check for --sort and --filter flags
		sortOrder := ""
		filterTag := ""
		for i := 2; i < len(os.Args); i++ {
			arg := os.Args[i]
			if len(arg) > 7 && arg[:7] == "--sort=" {
				sortOrder = arg[7:]
			}
			if len(arg) > 9 && arg[:9] == "--filter=" {
				filterTag = arg[9:]
			}
		}
		if err := commands.ListWithOptions(sortOrder, filterTag); err != nil {
			return handleError(err)
		}
		return 0

	case "--stats":
		if err := commands.Stats(); err != nil {
			return handleError(err)
		}
		return 0

	case "--list-aliases", "--names-only":
		// Hidden option for shell completion
		if err := commands.ListAliasNames(); err != nil {
			return handleError(err)
		}
		return 0

	case "--tags-raw":
		// Hidden option for shell completion - outputs just tag names
		if err := commands.ListTagsRaw(); err != nil {
			return handleError(err)
		}
		return 0

	case "-r", "--register":
		if len(os.Args) < 4 {
			fmt.Fprintln(os.Stderr, "Usage: goto -r <alias> <directory> [--tags=tag1,tag2]")
			return 1
		}
		// Check for --tags flag
		var tags []string
		for i := 4; i < len(os.Args); i++ {
			arg := os.Args[i]
			if len(arg) > 7 && arg[:7] == "--tags=" {
				tagStr := arg[7:]
				if tagStr != "" {
					tags = strings.Split(tagStr, ",")
				}
			}
		}
		if err := commands.RegisterWithTags(os.Args[2], os.Args[3], tags); err != nil {
			return handleError(err)
		}
		return 0

	case "-u", "--unregister":
		if len(os.Args) < 3 {
			fmt.Fprintln(os.Stderr, "Usage: goto -u <alias>")
			return 1
		}
		if err := commands.Unregister(os.Args[2]); err != nil {
			return handleError(err)
		}
		return 0

	case "-x", "--expand":
		if len(os.Args) < 3 {
			fmt.Fprintln(os.Stderr, "Usage: goto -x <alias>")
			return 1
		}
		if err := commands.Expand(os.Args[2]); err != nil {
			return handleError(err)
		}
		return 0

	case "-c", "--cleanup":
		if err := commands.Cleanup(); err != nil {
			return handleError(err)
		}
		return 0

	case "-p", "--push":
		if len(os.Args) < 3 {
			fmt.Fprintln(os.Stderr, "Usage: goto -p <alias>")
			return 1
		}
		if err := commands.Push(os.Args[2]); err != nil {
			return handleError(err)
		}
		return 0

	case "-o", "--pop":
		if err := commands.Pop(); err != nil {
			return handleError(err)
		}
		return 0

	case "--export":
		if err := commands.Export(); err != nil {
			return handleError(err)
		}
		return 0

	case "--rename":
		if len(os.Args) < 4 {
			fmt.Fprintln(os.Stderr, "Usage: goto --rename <old-alias> <new-alias>")
			return 1
		}
		if err := commands.Rename(os.Args[2], os.Args[3]); err != nil {
			return handleError(err)
		}
		return 0

	case "--tag":
		if len(os.Args) < 4 {
			fmt.Fprintln(os.Stderr, "Usage: goto --tag <alias> <tag>")
			return 1
		}
		if err := commands.AddTag(os.Args[2], os.Args[3]); err != nil {
			return handleError(err)
		}
		return 0

	case "--untag":
		if len(os.Args) < 4 {
			fmt.Fprintln(os.Stderr, "Usage: goto --untag <alias> <tag>")
			return 1
		}
		if err := commands.RemoveTag(os.Args[2], os.Args[3]); err != nil {
			return handleError(err)
		}
		return 0

	case "--tags":
		if err := commands.ListTags(); err != nil {
			return handleError(err)
		}
		return 0

	case "--recent":
		// Check if there's an argument (limit or index to navigate)
		if len(os.Args) >= 3 {
			arg := os.Args[2]
			n, err := strconv.Atoi(arg)
			if err != nil {
				fmt.Fprintf(os.Stderr, "Invalid argument: %s (expected number)\n", arg)
				return 1
			}
			// If n is small, treat as navigation to Nth recent
			// For display limits, user can use --recent with larger numbers
			if n >= 1 && n <= 20 {
				// Could be navigation or limit - use heuristic:
				// If more args or explicit indicator, it's a limit
				// Otherwise, navigate to Nth recent
				if len(os.Args) > 3 {
					// Extra args, show as list with limit
					if err := commands.ShowRecent(n); err != nil {
						return handleError(err)
					}
				} else {
					// Single number: navigate to Nth recent
					if err := commands.NavigateToRecent(n); err != nil {
						return handleError(err)
					}
				}
			} else {
				// Large number, treat as limit for display
				if err := commands.ShowRecent(n); err != nil {
					return handleError(err)
				}
			}
		} else {
			// No argument: show recent with default limit
			if err := commands.ShowRecent(10); err != nil {
				return handleError(err)
			}
		}
		return 0

	case "--recent-clear":
		if err := commands.ClearRecent(); err != nil {
			return handleError(err)
		}
		return 0

	case "--import":
		if len(os.Args) < 3 {
			fmt.Fprintln(os.Stderr, "Usage: goto --import <file> [--strategy=skip|overwrite|rename]")
			return 1
		}
		filepath := os.Args[2]
		strategy := "skip" // default strategy

		// Check for --strategy flag
		for i := 3; i < len(os.Args); i++ {
			arg := os.Args[i]
			if len(arg) > 11 && arg[:11] == "--strategy=" {
				strategy = arg[11:]
			}
		}

		result, err := commands.Import(filepath, strategy)
		if err != nil {
			return handleError(err)
		}

		// Print warnings
		for _, warning := range result.Warnings {
			fmt.Fprintln(os.Stderr, warning)
		}

		// Print summary
		fmt.Printf("Import complete: %d imported", result.Imported)
		if result.Skipped > 0 {
			fmt.Printf(", %d skipped", result.Skipped)
		}
		if result.Renamed > 0 {
			fmt.Printf(", %d renamed", result.Renamed)
		}
		fmt.Println()
		return 0

	default:
		// Default action: navigate to alias
		if arg[0] == '-' {
			fmt.Fprintf(os.Stderr, "Unknown option: %s\n", arg)
			return 1
		}
		if err := commands.Navigate(arg); err != nil {
			return handleError(err)
		}
		return 0
	}
}

func handleError(err error) int {
	fmt.Fprintln(os.Stderr, err)

	// Map errors to exit codes
	var notFound *alias.AliasNotFoundError
	var invalid *alias.InvalidAliasError
	var invalidTag *alias.InvalidTagError
	var exists *alias.AliasExistsError
	var dirNotFound *alias.DirectoryNotFoundError

	switch {
	case errors.As(err, &dirNotFound):
		return 2
	case errors.As(err, &invalid):
		return 3
	case errors.As(err, &invalidTag):
		return 3
	case errors.As(err, &exists):
		return 4
	case errors.As(err, &notFound):
		return 1
	case errors.Is(err, stack.ErrEmptyStack):
		return 1
	default:
		return 5
	}
}

func printUsage() {
	fmt.Println("Usage: goto <alias> or goto [OPTIONS]")
	fmt.Println("Try 'goto --help' for more information.")
}

func printHelp() {
	help := `goto-go - Navigate to aliased directories

Usage:
  goto <alias>                    Navigate to the directory
  goto -r <alias> <directory>     Register a new alias
  goto -r <alias> <dir> --tags=   Register with tags
  goto -u <alias>                 Unregister an alias
  goto -l                         List all aliases
  goto -l --sort=<order>          List aliases with sorting
  goto -l --filter=<tag>          List aliases with tag
  goto -x <alias>                 Expand alias to path
  goto -c                         Cleanup invalid aliases
  goto -p <alias>                 Push current dir, goto alias
  goto -o                         Pop and return to directory
  goto --rename <old> <new>       Rename an alias
  goto --tag <alias> <tag>        Add tag to alias
  goto --untag <alias> <tag>      Remove tag from alias
  goto --tags                     List all tags with counts
  goto --stats                    Show usage statistics
  goto --recent                   List recently visited directories
  goto --recent <N>               Navigate to Nth most recent
  goto --recent-clear             Clear recent history
  goto --export                   Export aliases to TOML (stdout)
  goto --import <file>            Import aliases from TOML file
  goto --config                   Show current configuration
  goto -v                         Show version
  goto -h                         Show this help

Sort options (use with -l/--list):
  --sort=alpha                    Sort alphabetically (default)
  --sort=usage                    Sort by use count (most used first)
  --sort=recent                   Sort by last used (most recent first)

Filter options (use with -l/--list):
  --filter=<tag>                  Show only aliases with tag

Import strategies (use with --import):
  --strategy=skip                 Skip existing aliases (default)
  --strategy=overwrite            Overwrite existing aliases
  --strategy=rename               Rename conflicting aliases (add suffix)

Tag rules:
  - Tags are case-insensitive (stored lowercase)
  - Tags must be alphanumeric with dash/underscore
  - No spaces in tags

Examples:
  goto -r dev ~/Development       Register 'dev' alias
  goto -r proj ~/code --tags=work,go  Register with tags
  goto dev                        Navigate to ~/Development
  goto -l --sort=usage            List aliases by usage
  goto -l --filter=work           List aliases tagged 'work'
  goto --tag dev golang           Add 'golang' tag to 'dev'
  goto --untag dev golang         Remove 'golang' tag from 'dev'
  goto --tags                     List all tags with counts
  goto --stats                    Show usage statistics
  goto --recent                   Show recently visited aliases
  goto --recent 3                 Navigate to 3rd most recent
  goto -p work                    Save location, go to 'work'
  goto -o                         Return to saved location
  goto --export > backup.toml     Backup aliases to file
  goto --import backup.toml       Restore aliases from backup
`
	fmt.Print(help)
}
