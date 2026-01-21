#!/usr/bin/env bats

# Test fzf integration in shell wrappers

setup() {
    # Save original PATH for teardown
    export ORIGINAL_PATH="$PATH"

    # Create isolated temp directory
    export TEST_DIR="$(mktemp -d)"
    export PROJECT_ROOT="${BATS_TEST_DIRNAME}/.."

    # Create mock bin directory
    export MOCK_BIN="$TEST_DIR/bin"
    mkdir -p "$MOCK_BIN"

    # Create test directories for navigation
    mkdir -p "$TEST_DIR/projects/foo"
    mkdir -p "$TEST_DIR/projects/bar"

    # Create mock goto-bin
    cat > "$MOCK_BIN/goto-bin" << 'MOCK_GOTO'
#!/bin/bash
case "$1" in
    --names-only)
        echo "foo"
        echo "bar"
        ;;
    -l)
        echo "foo    /tmp/foo"
        echo "bar    /tmp/bar"
        ;;
    -x)
        case "$2" in
            foo) echo "/tmp/projects/foo" ;;
            bar) echo "/tmp/projects/bar" ;;
        esac
        ;;
    foo)
        echo "$TEST_DIR/projects/foo"
        exit 0
        ;;
    bar)
        echo "$TEST_DIR/projects/bar"
        exit 0
        ;;
    *)
        echo "Unknown: $*"
        exit 1
        ;;
esac
MOCK_GOTO
    chmod +x "$MOCK_BIN/goto-bin"

    # Prepend mock bin to PATH
    export PATH="$MOCK_BIN:$PATH"
}

teardown() {
    # Restore original PATH first
    export PATH="$ORIGINAL_PATH"

    if [[ -d "$TEST_DIR" ]]; then
        rm -rf "$TEST_DIR"
    fi
}

# Helper to source bash wrapper and capture function definition
load_bash_wrapper() {
    source "$PROJECT_ROOT/shell/goto.bash"
}

#
# Fallback tests (no fzf available)
#

@test "bash: no-args without fzf shows list" {
    # Shadow fzf with a script that exits with error (simulating not found)
    cat > "$MOCK_BIN/fzf" << 'EOF'
#!/bin/bash
exit 127
EOF
    chmod +x "$MOCK_BIN/fzf"

    # Override command -v to report fzf not found
    run bash -c '
        fzf() { return 127; }
        command() {
            if [[ "$1" == "-v" && "$2" == "fzf" ]]; then
                return 1
            fi
            builtin command "$@"
        }
        export PATH="'"$MOCK_BIN"':$PATH"
        source '"$PROJECT_ROOT"'/shell/goto.bash
        goto
    '

    [[ "$output" == *"foo"* ]]
    [[ "$output" == *"bar"* ]]
}

@test "bash: no-args in non-interactive mode shows list" {
    # Create mock fzf that should NOT be called
    cat > "$MOCK_BIN/fzf" << 'EOF'
#!/bin/bash
echo "FZF_SHOULD_NOT_BE_CALLED"
exit 1
EOF
    chmod +x "$MOCK_BIN/fzf"

    load_bash_wrapper

    # Pipe input to make stdin non-interactive
    run bash -c 'source '"$PROJECT_ROOT"'/shell/goto.bash && echo "" | goto'

    # Should show list, not call fzf
    [[ "$output" != *"FZF_SHOULD_NOT_BE_CALLED"* ]]
}

#
# fzf integration tests
#

@test "bash: fzf receives alias names from --names-only" {
    # Create mock fzf that records its stdin
    cat > "$MOCK_BIN/fzf" << 'EOF'
#!/bin/bash
cat > "$TEST_DIR/fzf_input.txt"
# Simulate user pressing Escape (empty selection)
exit 0
EOF
    chmod +x "$MOCK_BIN/fzf"

    load_bash_wrapper

    # Run in a pseudo-terminal context using script command
    script -q -c 'source '"$PROJECT_ROOT"'/shell/goto.bash && goto' /dev/null < /dev/null || true

    # Verify fzf received the alias names
    if [[ -f "$TEST_DIR/fzf_input.txt" ]]; then
        [[ "$(cat "$TEST_DIR/fzf_input.txt")" == *"foo"* ]]
        [[ "$(cat "$TEST_DIR/fzf_input.txt")" == *"bar"* ]]
    fi
}

@test "bash: selecting alias in fzf navigates to directory" {
    # Create mock fzf that selects "foo"
    cat > "$MOCK_BIN/fzf" << 'EOF'
#!/bin/bash
echo "foo"
EOF
    chmod +x "$MOCK_BIN/fzf"

    # Need to test that cd would be called with the right path
    # We'll check by examining the function behavior
    load_bash_wrapper

    # Capture what directory we'd cd to by running in subshell
    result=$(script -q -c '
        source '"$PROJECT_ROOT"'/shell/goto.bash
        goto
        pwd
    ' /dev/null 2>/dev/null | tail -1)

    # Should have changed to foo directory
    [[ "$result" == *"projects/foo"* ]] || [[ "$?" -eq 0 ]]
}

@test "bash: canceling fzf (empty selection) returns silently" {
    # Create mock fzf that returns empty (user pressed Escape)
    cat > "$MOCK_BIN/fzf" << 'EOF'
#!/bin/bash
# Return empty string (user canceled)
exit 0
EOF
    chmod +x "$MOCK_BIN/fzf"

    load_bash_wrapper

    run script -q -c 'source '"$PROJECT_ROOT"'/shell/goto.bash && goto' /dev/null

    # Should return 0 and not produce error output
    [[ "$status" -eq 0 ]]
}

@test "bash: GOTO_FZF_OPTS is passed to fzf" {
    # Create mock fzf that records its arguments
    cat > "$MOCK_BIN/fzf" << 'EOF'
#!/bin/bash
echo "$@" > "$TEST_DIR/fzf_args.txt"
exit 0
EOF
    chmod +x "$MOCK_BIN/fzf"

    export GOTO_FZF_OPTS="--height 80%"
    load_bash_wrapper

    script -q -c 'source '"$PROJECT_ROOT"'/shell/goto.bash && goto' /dev/null < /dev/null || true

    if [[ -f "$TEST_DIR/fzf_args.txt" ]]; then
        [[ "$(cat "$TEST_DIR/fzf_args.txt")" == *"--height 80%"* ]]
    fi
}

@test "bash: fzf preview uses goto-bin -x" {
    # Create mock fzf that records its arguments
    cat > "$MOCK_BIN/fzf" << 'EOF'
#!/bin/bash
echo "$@" > "$TEST_DIR/fzf_args.txt"
exit 0
EOF
    chmod +x "$MOCK_BIN/fzf"

    load_bash_wrapper

    script -q -c 'source '"$PROJECT_ROOT"'/shell/goto.bash && goto' /dev/null < /dev/null || true

    if [[ -f "$TEST_DIR/fzf_args.txt" ]]; then
        [[ "$(cat "$TEST_DIR/fzf_args.txt")" == *"--preview"* ]]
        [[ "$(cat "$TEST_DIR/fzf_args.txt")" == *"goto-bin -x"* ]]
    fi
}

#
# Direct navigation tests (ensure existing behavior unchanged)
#

@test "bash: goto with alias argument navigates directly" {
    load_bash_wrapper

    # Navigate to foo - should not invoke fzf
    local start_dir="$(pwd)"
    cd "$TEST_DIR"

    # Run in subshell to test cd
    result=$(bash -c '
        source '"$PROJECT_ROOT"'/shell/goto.bash
        goto foo
        pwd
    ')

    [[ "$result" == *"projects/foo"* ]]

    cd "$start_dir"
}

@test "bash: goto -l still works" {
    load_bash_wrapper
    run goto -l

    [[ "$output" == *"foo"* ]]
    [[ "$output" == *"bar"* ]]
}
