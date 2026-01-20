#!/usr/bin/env bats

# Test uninstall dry-run output for mise run uninstall -- --dry-run

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
}

teardown() {
    # Restore original HOME
    export HOME="$ORIGINAL_HOME"

    # Clean up temp directory
    if [[ -d "$TEST_HOME" ]]; then
        rm -rf "$TEST_HOME"
    fi
}

# Helper to install goto first (needed for most uninstall tests)
install_goto() {
    cd "$PROJECT_ROOT"
    mise run install -- --shell=bash
}

@test "dry run shows all 4 steps" {
    cd "$PROJECT_ROOT"
    run mise run uninstall -- --dry-run

    [[ "$output" == *"[1/4]"* ]]
    [[ "$output" == *"[2/4]"* ]]
    [[ "$output" == *"[3/4]"* ]]
    [[ "$output" == *"[4/4]"* ]]
}

@test "dry run shows 'Would remove' for existing files" {
    # First install goto so there are files to remove
    install_goto

    cd "$PROJECT_ROOT"
    run mise run uninstall -- --dry-run

    [[ "$output" == *"Would remove"* ]]
}

@test "dry run ends with no-changes message" {
    cd "$PROJECT_ROOT"
    run mise run uninstall -- --dry-run

    [[ "$output" == *"Dry run complete. No changes were made."* ]]
}

@test "dry run shows files to clean from rc files" {
    # First install goto to add source line to bashrc
    install_goto

    cd "$PROJECT_ROOT"
    run mise run uninstall -- --dry-run

    [[ "$output" == *"Would remove goto lines from"* ]]
}

@test "unknown option fails" {
    cd "$PROJECT_ROOT"
    run mise run uninstall -- --unknown

    [[ "$status" -ne 0 ]]
    [[ "$output" == *"Unknown option"* ]]
}
