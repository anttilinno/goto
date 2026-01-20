#!/usr/bin/env bats

# Test install error handling

setup() {
    # Create isolated temp HOME directory
    export ORIGINAL_HOME="$HOME"
    export TEST_HOME="$(mktemp -d)"
    export HOME="$TEST_HOME"

    # Store project root for mise
    export PROJECT_ROOT="${BATS_TEST_DIRNAME}/.."

    # Trust mise config when running with different HOME
    export MISE_TRUSTED_CONFIG_PATHS="$PROJECT_ROOT"
}

teardown() {
    # Restore original HOME
    export HOME="$ORIGINAL_HOME"

    # Clean up temp directory
    if [[ -d "$TEST_HOME" ]]; then
        rm -rf "$TEST_HOME"
    fi
}

@test "invalid shell type fails with error" {
    cd "$PROJECT_ROOT"
    run mise run install -- --shell=invalid

    [[ "$status" -ne 0 ]]
    [[ "$output" == *"Invalid shell type"* ]]
    [[ "$output" == *"bash, zsh, or fish"* ]]
}

@test "unknown option fails with error" {
    cd "$PROJECT_ROOT"
    run mise run install -- --unknown-flag

    [[ "$status" -ne 0 ]]
    [[ "$output" == *"Unknown option"* ]]
    [[ "$output" == *"Usage:"* ]]
}

@test "invalid SHELL env fails when no --shell flag provided" {
    cd "$PROJECT_ROOT"
    export SHELL="/bin/tcsh"
    run mise run install -- --dry-run

    [[ "$status" -ne 0 ]]
    [[ "$output" == *"Could not auto-detect shell"* ]]
    [[ "$output" == *"--shell=bash|zsh|fish"* ]]
}
