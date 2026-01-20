#!/usr/bin/env bats

# Test install dry-run output for mise run install -- --dry-run

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

@test "dry run shows all 5 steps" {
    cd "$PROJECT_ROOT"
    run mise run install -- --dry-run --shell=bash

    [[ "$output" == *"[1/5]"* ]]
    [[ "$output" == *"[2/5]"* ]]
    [[ "$output" == *"[3/5]"* ]]
    [[ "$output" == *"[4/5]"* ]]
    [[ "$output" == *"[5/5]"* ]]
}

@test "dry run shows 'Would' actions" {
    cd "$PROJECT_ROOT"
    run mise run install -- --dry-run --shell=bash

    [[ "$output" == *"Would create"* ]] || [[ "$output" == *"Would copy"* ]]
}

@test "dry run ends with no-changes message" {
    cd "$PROJECT_ROOT"
    run mise run install -- --dry-run --shell=bash

    [[ "$output" == *"Dry run complete. No changes were made."* ]]
}

@test "dry run does not modify filesystem" {
    cd "$PROJECT_ROOT"

    # Ensure directories don't exist before
    [[ ! -d "$TEST_HOME/.local/bin" ]]
    [[ ! -d "$TEST_HOME/.config/goto" ]]

    # Run dry-run install
    run mise run install -- --dry-run --shell=bash

    # Verify no files were created
    [[ ! -f "$TEST_HOME/.local/bin/goto-bin" ]]
    [[ ! -d "$TEST_HOME/.config/goto" ]]

    # Verify no goto entries in bashrc (if it exists)
    if [[ -f "$TEST_HOME/.bashrc" ]]; then
        ! grep -q "goto" "$TEST_HOME/.bashrc"
    fi
}
