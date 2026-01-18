# goto-go: Tool Specification

A Go rewrite of [iridakos/goto](https://github.com/iridakos/goto) - a shell utility for navigating to aliased directories with tab completion.

---

## 1. Overview

**goto** is a command-line tool that allows users to register aliases for directories and quickly navigate to them. The Go implementation should provide a single binary with shell integration scripts for bash, zsh, and optionally fish.

### Goals
- Single static binary (easy distribution)
- Cross-platform support (Linux, macOS, Windows)
- Shell completion for bash, zsh, and fish
- Backward compatibility with existing goto database format
- Improved performance over shell script implementation

---

## 2. Command-Line Interface

### 2.1 Synopsis

```
goto [OPTIONS] <alias>
goto -r|--register <alias> <directory>
goto -u|--unregister <alias>
goto -l|--list
goto -x|--expand <alias>
goto -c|--cleanup
goto -p|--push <alias>
goto -o|--pop
goto -v|--version
goto -h|--help
```

### 2.2 Commands & Options

| Option | Long Form | Arguments | Description |
|--------|-----------|-----------|-------------|
| (none) | | `<alias>` | Navigate to the directory associated with the alias |
| `-r` | `--register` | `<alias> <directory>` | Register a new alias for a directory |
| `-u` | `--unregister` | `<alias>` | Remove an existing alias |
| `-l` | `--list` | | List all registered aliases |
| `-x` | `--expand` | `<alias>` | Print the directory path for an alias |
| `-c` | `--cleanup` | | Remove aliases pointing to non-existent directories |
| `-p` | `--push` | `<alias>` | Push current directory to stack, then goto alias |
| `-o` | `--pop` | | Pop directory from stack and navigate to it |
| `-v` | `--version` | | Display version information |
| `-h` | `--help` | | Display help information |

### 2.3 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (invalid arguments, alias not found, etc.) |
| 2 | Directory does not exist |
| 3 | Invalid alias name |
| 4 | Alias already exists (on register) |
| 5 | Database read/write error |

---

## 3. Alias Rules

### 3.1 Valid Alias Names
- Must start with a letter (a-z, A-Z) or digit (0-9)
- Can contain letters, digits, hyphens (`-`), and underscores (`_`)
- Case-sensitive
- Regex pattern: `^[a-zA-Z0-9][a-zA-Z0-9_-]*$`

### 3.2 Directory Handling
- Directories should be expanded to absolute paths when registered
- Support for `.` (current directory) expansion
- Support for `~` (home directory) expansion
- Support for environment variables in paths (e.g., `$HOME/projects`)

---

## 4. Data Storage

### 4.1 Database Location

Priority order:
1. `$GOTO_DB` environment variable (if set)
2. `$XDG_CONFIG_HOME/goto` (if `$XDG_CONFIG_HOME` is set)
3. `~/.config/goto` (default)

**Note:** For backward compatibility with v1.x, optionally check for `~/.goto` and migrate.

### 4.2 Database Format

Simple text file with one alias per line:

```
<alias> <absolute-path>
```

Example:
```
dev /home/user/development
blog /var/www/html/blog
dotfiles /home/user/.dotfiles
```

**Format Rules:**
- Space-separated (first space is delimiter)
- Paths may contain spaces (everything after first space is the path)
- Lines starting with `#` are comments (optional enhancement)
- Empty lines are ignored

### 4.3 Directory Stack

For push/pop functionality, maintain a stack file:
- Location: Same directory as database, named `goto_stack` or `.goto_stack`
- Format: One directory path per line (stack top at end of file)

---

## 5. Core Functions

### 5.1 Navigate (default action)

```go
func Navigate(alias string) error
```

1. Look up alias in database
2. Verify directory exists
3. Output the path (shell wrapper performs actual `cd`)

**Output:** Print path to stdout for shell wrapper to use

### 5.2 Register

```go
func Register(alias, directory string) error
```

1. Validate alias name format
2. Expand directory to absolute path
3. Verify directory exists
4. Check alias doesn't already exist
5. Append to database file

**Output:** Success message or error

### 5.3 Unregister

```go
func Unregister(alias string) error
```

1. Verify alias exists
2. Remove from database file

**Output:** Success message or error

### 5.4 List

```go
func List() error
```

1. Read all aliases from database
2. Format as aligned columns

**Output:**
```
alias1    /path/to/directory1
alias2    /path/to/directory2
```

### 5.5 Expand

```go
func Expand(alias string) error
```

1. Look up alias
2. Print path

**Output:** Just the path (no decoration)

### 5.6 Cleanup

```go
func Cleanup() error
```

1. Read all aliases
2. Check each directory exists
3. Remove entries for non-existent directories
4. Report what was removed

**Output:** List of removed aliases or "Nothing to clean up"

### 5.7 Push

```go
func Push(alias string) error
```

1. Get current working directory
2. Push to stack file
3. Perform navigate

### 5.8 Pop

```go
func Pop() error
```

1. Pop from stack file
2. Output directory for shell to cd

---

## 6. Shell Integration

Since a binary cannot change the shell's working directory directly, shell wrapper functions are required.

### 6.1 Bash Integration (`goto.bash`)

```bash
# goto shell wrapper for bash
goto() {
    local output
    output=$(goto-bin "$@")
    local exit_code=$?
    
    case "$1" in
        ""|-h|--help|-v|--version|-l|--list|-c|--cleanup|-x|--expand)
            echo "$output"
            ;;
        -r|--register|-u|--unregister)
            echo "$output"
            ;;
        *)
            if [[ $exit_code -eq 0 && -n "$output" && -d "$output" ]]; then
                cd "$output" || return 1
            else
                echo "$output"
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
    
    case "$prev" in
        -r|--register)
            # Complete with directory names
            COMPREPLY=($(compgen -d -- "$cur"))
            ;;
        -u|--unregister|-x|--expand|-p|--push)
            # Complete with alias names
            COMPREPLY=($(compgen -W "$(goto-bin --list-aliases)" -- "$cur"))
            ;;
        goto)
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "-r --register -u --unregister -l --list -x --expand -c --cleanup -p --push -o --pop -v --version -h --help" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "$(goto-bin --list-aliases)" -- "$cur"))
            fi
            ;;
        *)
            COMPREPLY=($(compgen -W "$(goto-bin --list-aliases)" -- "$cur"))
            ;;
    esac
}

complete -F _goto_completions goto
```

### 6.2 Zsh Integration (`goto.zsh`)

```zsh
# goto shell wrapper for zsh
goto() {
    local output
    output=$(goto-bin "$@")
    local exit_code=$?
    
    case "$1" in
        ""|-h|--help|-v|--version|-l|--list|-c|--cleanup|-x|--expand)
            echo "$output"
            ;;
        -r|--register|-u|--unregister)
            echo "$output"
            ;;
        *)
            if [[ $exit_code -eq 0 && -n "$output" && -d "$output" ]]; then
                cd "$output" || return 1
            else
                echo "$output"
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
    
    options=(
        '-r[Register an alias]:alias:->register'
        '--register[Register an alias]:alias:->register'
        '-u[Unregister an alias]:alias:->aliases'
        '--unregister[Unregister an alias]:alias:->aliases'
        '-l[List all aliases]'
        '--list[List all aliases]'
        '-x[Expand an alias]:alias:->aliases'
        '--expand[Expand an alias]:alias:->aliases'
        '-c[Cleanup invalid aliases]'
        '--cleanup[Cleanup invalid aliases]'
        '-p[Push current dir and goto]:alias:->aliases'
        '--push[Push current dir and goto]:alias:->aliases'
        '-o[Pop and go to directory]'
        '--pop[Pop and go to directory]'
        '-v[Show version]'
        '--version[Show version]'
        '-h[Show help]'
        '--help[Show help]'
    )
    
    _arguments -s $options '*:alias:->aliases'
    
    case "$state" in
        aliases)
            aliases=(${(f)"$(goto-bin --list-aliases)"})
            _describe 'alias' aliases
            ;;
        register)
            _files -/
            ;;
    esac
}

compdef _goto goto
```

### 6.3 Fish Integration (`goto.fish`)

```fish
# goto shell wrapper for fish
function goto
    set -l output (goto-bin $argv)
    set -l exit_code $status
    
    switch "$argv[1]"
        case "" -h --help -v --version -l --list -c --cleanup -x --expand -r --register -u --unregister
            echo $output
        case '*'
            if test $exit_code -eq 0 -a -n "$output" -a -d "$output"
                cd $output
            else
                echo $output
                return $exit_code
            end
    end
    return $exit_code
end

# Fish completions
complete -c goto -f
complete -c goto -n "not __fish_seen_subcommand_from -r --register -u --unregister -l --list -x --expand -c --cleanup -p --push -o --pop -v --version -h --help" -a "(goto-bin --list-aliases)"
complete -c goto -s r -l register -d "Register alias" -r -F
complete -c goto -s u -l unregister -d "Unregister alias" -a "(goto-bin --list-aliases)"
complete -c goto -s l -l list -d "List aliases"
complete -c goto -s x -l expand -d "Expand alias" -a "(goto-bin --list-aliases)"
complete -c goto -s c -l cleanup -d "Cleanup invalid aliases"
complete -c goto -s p -l push -d "Push and goto" -a "(goto-bin --list-aliases)"
complete -c goto -s o -l pop -d "Pop directory"
complete -c goto -s v -l version -d "Show version"
complete -c goto -s h -l help -d "Show help"
```

### 6.4 Hidden Option for Completions

Add a hidden `--list-aliases` option that outputs only alias names (one per line) for shell completion scripts:

```
goto-bin --list-aliases
```

Output:
```
dev
blog
dotfiles
```

---

## 7. Project Structure

```
goto-go/
├── cmd/
│   └── goto/
│       └── main.go           # Entry point
├── internal/
│   ├── config/
│   │   └── config.go         # Configuration & paths
│   ├── database/
│   │   └── database.go       # Alias storage operations
│   ├── stack/
│   │   └── stack.go          # Directory stack operations
│   └── alias/
│       └── alias.go          # Alias validation
├── shell/
│   ├── goto.bash             # Bash integration
│   ├── goto.zsh              # Zsh integration
│   └── goto.fish             # Fish integration
├── scripts/
│   └── install.sh            # Installation script
├── go.mod
├── go.sum
├── Makefile
├── README.md
└── LICENSE
```

---

## 8. Go Implementation Notes

### 8.1 Recommended Libraries

- **CLI Parsing:** `github.com/spf13/cobra` or standard `flag` package
- **Home Directory:** `os.UserHomeDir()` (Go 1.12+)
- **Path Operations:** `path/filepath` package
- **File Locking:** `github.com/gofrs/flock` (for concurrent access)

### 8.2 Key Types

```go
// Alias represents a directory alias
type Alias struct {
    Name string
    Path string
}

// Database handles alias persistence
type Database struct {
    path string
}

// Stack handles the directory stack
type Stack struct {
    path string
}
```

### 8.3 Error Handling

Use custom error types for better error messages:

```go
type AliasNotFoundError struct {
    Alias string
}

type InvalidAliasError struct {
    Alias  string
    Reason string
}

type DirectoryNotFoundError struct {
    Path string
}

type AliasExistsError struct {
    Alias string
}
```

---

## 9. Installation

### 9.1 Install Script

The install script should:
1. Copy binary to `/usr/local/bin/goto-bin` (or user-specified location)
2. Copy shell scripts to appropriate locations
3. Detect user's shell and add source line to shell rc file
4. Provide instructions for manual installation

### 9.2 Homebrew Formula (Future)

```ruby
class GotoGo < Formula
  desc "Navigate to aliased directories with tab completion"
  homepage "https://github.com/yourusername/goto-go"
  url "https://github.com/yourusername/goto-go/archive/v1.0.0.tar.gz"
  sha256 "..."
  license "MIT"

  depends_on "go" => :build

  def install
    system "go", "build", "-o", bin/"goto-bin", "./cmd/goto"
    bash_completion.install "shell/goto.bash"
    zsh_completion.install "shell/goto.zsh" => "_goto"
    fish_completion.install "shell/goto.fish"
  end
end
```

---

## 10. Testing

### 10.1 Unit Tests

- Alias validation (valid/invalid names)
- Database read/write operations
- Path expansion (`~`, `.`, environment variables)
- Stack operations

### 10.2 Integration Tests

- Full workflow: register → list → navigate → unregister
- Cleanup with mix of valid/invalid directories
- Push/pop operations
- Edge cases (empty database, special characters in paths)

### 10.3 Test Commands

```bash
# Run all tests
go test ./...

# Run with coverage
go test -cover ./...

# Run specific package
go test ./internal/database
```

---

## 11. Enhancements (Optional)

These are potential improvements over the original:

1. **Import/Export:** `goto --export > aliases.txt` and `goto --import aliases.txt`
2. **Fuzzy Matching:** Partial alias matching when exact match not found
3. **Alias Rename:** `goto --rename <old> <new>`
4. **Usage Stats:** Track and display most-used aliases
5. **Sync:** Cloud sync support for aliases across machines
6. **Tags/Groups:** Organize aliases into categories
7. **Recent:** Show recently visited directories
8. **Config File:** Support for additional configuration options

---

## 12. Version & License

- Initial version: 1.0.0
- License: MIT (same as original)
- Maintain attribution to original project

---

## 13. References

- Original Project: https://github.com/iridakos/goto
- Original Author: Lazarus Lazaridis (iridakos)
- Blog Post: https://iridakos.com/programming/2019/04/10/shell-navigation-with-autocomplete
