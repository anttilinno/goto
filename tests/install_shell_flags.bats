#!/usr/bin/env bats

# Test install shell flag variations

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

@test "--shell=bash uses .bashrc" {
    cd "$PROJECT_ROOT"
    run mise run install -- --dry-run --shell=bash

    [[ "$output" == *".bashrc"* ]]
    [[ "$output" == *"goto.bash"* ]]
}

@test "--shell=zsh uses .zshrc" {
    cd "$PROJECT_ROOT"
    run mise run install -- --dry-run --shell=zsh

    [[ "$output" == *".zshrc"* ]]
    [[ "$output" == *"goto.zsh"* ]]
}

@test "--shell=fish uses config.fish" {
    cd "$PROJECT_ROOT"
    run mise run install -- --dry-run --shell=fish

    [[ "$output" == *".config/fish/config.fish"* ]]
    [[ "$output" == *"goto.fish"* ]]
}

@test "auto-detect from SHELL=bash" {
    cd "$PROJECT_ROOT"
    export SHELL="/bin/bash"
    run mise run install -- --dry-run

    [[ "$output" == *".bashrc"* ]]
    [[ "$output" == *"goto.bash"* ]]
}

@test "auto-detect from SHELL=zsh" {
    cd "$PROJECT_ROOT"
    export SHELL="/bin/zsh"
    run mise run install -- --dry-run

    [[ "$output" == *".zshrc"* ]]
    [[ "$output" == *"goto.zsh"* ]]
}
