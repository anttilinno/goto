#!/usr/bin/env bats

# Test that uninstall preserves user data while removing installation files

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

    # Define paths for convenience
    export CONFIG_DIR="$TEST_HOME/.config/goto"
    export BIN_DIR="$TEST_HOME/.local/bin"
}

teardown() {
    # Restore original HOME
    export HOME="$ORIGINAL_HOME"

    # Clean up temp directory
    if [[ -d "$TEST_HOME" ]]; then
        rm -rf "$TEST_HOME"
    fi
}

# Helper to install goto first
install_goto() {
    cd "$PROJECT_ROOT"
    mise run install -- --shell=bash
}

# Helper to create a test aliases.toml file
create_aliases_file() {
    mkdir -p "$CONFIG_DIR"
    cat > "$CONFIG_DIR/aliases.toml" << 'EOF'
[aliases]
[aliases.proj]
path = "/home/user/projects"
tags = []
use_count = 5
EOF
}

@test "preserves aliases.toml after uninstall" {
    # Install goto first
    install_goto

    # Create user's aliases file
    create_aliases_file

    # Run actual uninstall
    cd "$PROJECT_ROOT"
    run mise run uninstall

    # Verify aliases.toml still exists
    [[ -f "$CONFIG_DIR/aliases.toml" ]]
}

@test "shows preservation notice for aliases.toml" {
    # Install goto first
    install_goto

    # Create user's aliases file
    create_aliases_file

    # Run uninstall
    cd "$PROJECT_ROOT"
    run mise run uninstall

    # Verify output contains preservation message
    [[ "$output" == *"Preserving"* ]]
    [[ "$output" == *"aliases.toml"* ]]
}

@test "removes shell wrappers" {
    # Install goto first
    install_goto

    # Create aliases file to prevent config dir removal
    create_aliases_file

    # Verify wrapper was installed
    [[ -f "$CONFIG_DIR/goto.bash" ]]

    # Run actual uninstall
    cd "$PROJECT_ROOT"
    run mise run uninstall

    # Verify shell wrappers are removed
    [[ ! -f "$CONFIG_DIR/goto.bash" ]]
    [[ ! -f "$CONFIG_DIR/goto.zsh" ]]
    [[ ! -f "$CONFIG_DIR/goto.fish" ]]
}

@test "removes binary" {
    # Install goto first
    install_goto

    # Verify binary was installed
    [[ -f "$BIN_DIR/goto-bin" ]]

    # Run actual uninstall
    cd "$PROJECT_ROOT"
    run mise run uninstall

    # Verify binary is removed
    [[ ! -f "$BIN_DIR/goto-bin" ]]
}

@test "cleans empty config dir when no aliases.toml exists" {
    # Install goto first
    install_goto

    # Verify config dir exists with wrapper
    [[ -d "$CONFIG_DIR" ]]
    [[ -f "$CONFIG_DIR/goto.bash" ]]

    # Do NOT create aliases.toml - leave only shell wrapper

    # Run actual uninstall
    cd "$PROJECT_ROOT"
    run mise run uninstall

    # Verify config directory is removed (since it's empty after removing wrapper)
    [[ ! -d "$CONFIG_DIR" ]]
}

@test "config dir persists when aliases.toml exists" {
    # Install goto first
    install_goto

    # Create user's aliases file
    create_aliases_file

    # Run actual uninstall
    cd "$PROJECT_ROOT"
    run mise run uninstall

    # Config dir should still exist because aliases.toml is preserved
    [[ -d "$CONFIG_DIR" ]]
}

@test "removes source line from bashrc" {
    # Install goto first
    install_goto

    # Verify source line was added
    grep -q "source.*goto.bash" "$TEST_HOME/.bashrc"

    # Run actual uninstall
    cd "$PROJECT_ROOT"
    run mise run uninstall

    # Verify source line is removed
    ! grep -q "source.*goto.bash" "$TEST_HOME/.bashrc"
}

@test "removes goto comment from bashrc" {
    # Install goto first
    install_goto

    # Verify comment was added
    grep -q "# goto - directory navigation" "$TEST_HOME/.bashrc"

    # Run actual uninstall
    cd "$PROJECT_ROOT"
    run mise run uninstall

    # Verify comment is removed
    ! grep -q "# goto - directory navigation" "$TEST_HOME/.bashrc"
}
