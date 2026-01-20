#!/usr/bin/env bats

# Test install --bin-dir flag variations

setup() {
    # Create isolated temp HOME directory
    export ORIGINAL_HOME="$HOME"
    export TEST_HOME="$(mktemp -d)"
    export HOME="$TEST_HOME"

    # Set predictable shell
    export SHELL="/bin/bash"

    # Store project root for mise
    export PROJECT_ROOT="${BATS_TEST_DIRNAME}/.."

    # Trust mise config when running with different HOME
    export MISE_TRUSTED_CONFIG_PATHS="$PROJECT_ROOT"

    # Save original PATH
    export ORIGINAL_PATH="$PATH"
}

teardown() {
    # Restore original HOME
    export HOME="$ORIGINAL_HOME"

    # Restore original PATH
    export PATH="$ORIGINAL_PATH"

    # Clean up temp directory
    if [[ -d "$TEST_HOME" ]]; then
        rm -rf "$TEST_HOME"
    fi
}

@test "custom bin-dir appears in output" {
    cd "$PROJECT_ROOT"
    run mise run install -- --dry-run --shell=bash --bin-dir=/custom/path

    [[ "$output" == *"/custom/path/goto-bin"* ]]
}

@test "tilde expansion works in bin-dir" {
    cd "$PROJECT_ROOT"
    run mise run install -- --dry-run --shell=bash --bin-dir=~/mybin

    [[ "$output" == *"$TEST_HOME/mybin/goto-bin"* ]]
}

@test "PATH warning when bin-dir not in PATH" {
    cd "$PROJECT_ROOT"
    # Use a path that's definitely not in PATH
    run mise run install -- --dry-run --shell=bash --bin-dir=/nonexistent/custom/bin

    [[ "$output" == *"WARNING"* ]]
    [[ "$output" == *"/nonexistent/custom/bin is not in your PATH"* ]]
}

@test "no PATH warning when bin-dir is in PATH" {
    cd "$PROJECT_ROOT"
    # Create a temp bin dir and add it to PATH
    local temp_bin="$TEST_HOME/in-path-bin"
    mkdir -p "$temp_bin"
    export PATH="$temp_bin:$PATH"

    run mise run install -- --dry-run --shell=bash --bin-dir="$temp_bin"

    # Should show "is in PATH" message, not WARNING
    [[ "$output" == *"$temp_bin is in PATH"* ]]
    [[ "$output" != *"WARNING"* ]]
}
