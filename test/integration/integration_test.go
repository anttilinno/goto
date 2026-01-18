package integration

import (
	"fmt"
	"os"
	"strings"
	"testing"
)

func TestBinaryExists(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	stdout, _, exitCode := env.Run("--version")
	if exitCode != 0 {
		t.Fatalf("Expected exit 0, got %d", exitCode)
	}
	if stdout == "" {
		t.Error("Expected version output")
	}
}

// Register tests

func TestRegister(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	testDir := env.CreateTestDir("myproject")

	// Register alias
	stdout := env.MustRun("-r", "proj", testDir)
	if !strings.Contains(stdout, "Registered") {
		t.Errorf("Expected 'Registered' message, got: %s", stdout)
	}

	// Verify it appears in list
	stdout = env.MustRun("-l")
	if !strings.Contains(stdout, "proj") {
		t.Errorf("Expected 'proj' in list, got: %s", stdout)
	}
	if !strings.Contains(stdout, testDir) {
		t.Errorf("Expected path in list, got: %s", stdout)
	}
}

func TestRegisterCurrentDir(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	testDir := env.CreateTestDir("current")

	// Change to test dir and register with "."
	origDir, _ := os.Getwd()
	os.Chdir(testDir)
	defer os.Chdir(origDir)

	env.MustRun("-r", "here", ".")

	// Expand should return absolute path
	stdout := env.MustRun("-x", "here")
	if stdout != testDir {
		t.Errorf("Expected %s, got %s", testDir, stdout)
	}
}

func TestRegisterDuplicate(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	testDir := env.CreateTestDir("dup")
	env.MustRun("-r", "dup", testDir)

	// Try to register same alias again
	_, stderr, exitCode := env.Run("-r", "dup", testDir)
	if exitCode != 4 {
		t.Errorf("Expected exit code 4 (alias exists), got %d", exitCode)
	}
	if !strings.Contains(stderr, "already exists") {
		t.Errorf("Expected 'already exists' error, got: %s", stderr)
	}
}

func TestRegisterInvalidAlias(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	testDir := env.CreateTestDir("test")

	tests := []struct {
		alias string
		desc  string
	}{
		{"-invalid", "starts with hyphen"},
		{"_invalid", "starts with underscore"},
		{"has space", "contains space"},
		{"has.dot", "contains dot"},
	}

	for _, tt := range tests {
		_, _, exitCode := env.Run("-r", tt.alias, testDir)
		if exitCode != 3 {
			t.Errorf("%s: expected exit code 3, got %d", tt.desc, exitCode)
		}
	}
}

func TestRegisterNonexistentDir(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	_, stderr, exitCode := env.Run("-r", "ghost", "/nonexistent/path")
	if exitCode != 2 {
		t.Errorf("Expected exit code 2 (dir not found), got %d", exitCode)
	}
	if !strings.Contains(stderr, "does not exist") {
		t.Errorf("Expected 'does not exist' error, got: %s", stderr)
	}
}

// Unregister tests

func TestUnregister(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	testDir := env.CreateTestDir("tounreg")
	env.MustRun("-r", "unreg", testDir)

	// Unregister
	stdout := env.MustRun("-u", "unreg")
	if !strings.Contains(stdout, "Unregistered") {
		t.Errorf("Expected 'Unregistered' message, got: %s", stdout)
	}

	// Verify it's gone
	stdout = env.MustRun("-l")
	if strings.Contains(stdout, "unreg") {
		t.Error("Alias should not appear in list after unregister")
	}
}

func TestUnregisterNotFound(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	_, stderr, exitCode := env.Run("-u", "doesnotexist")
	if exitCode != 1 {
		t.Errorf("Expected exit code 1, got %d", exitCode)
	}
	if !strings.Contains(stderr, "not found") {
		t.Errorf("Expected 'not found' error, got: %s", stderr)
	}
}

// Navigate tests

func TestNavigate(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	testDir := env.CreateTestDir("navtest")
	env.MustRun("-r", "nav", testDir)

	// Navigate should output the path
	stdout := env.MustRun("nav")
	if stdout != testDir {
		t.Errorf("Expected %s, got %s", testDir, stdout)
	}
}

func TestNavigateNotFound(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	_, stderr, exitCode := env.Run("nonexistent")
	if exitCode != 1 {
		t.Errorf("Expected exit code 1, got %d", exitCode)
	}
	if !strings.Contains(stderr, "not found") {
		t.Errorf("Expected 'not found' error, got: %s", stderr)
	}
}

func TestNavigateDeletedDirectory(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	testDir := env.CreateTestDir("willdelete")
	env.MustRun("-r", "deleted", testDir)

	// Delete the directory
	os.RemoveAll(testDir)

	// Navigate should fail
	_, stderr, exitCode := env.Run("deleted")
	if exitCode != 2 {
		t.Errorf("Expected exit code 2 (dir not found), got %d", exitCode)
	}
	if !strings.Contains(stderr, "does not exist") {
		t.Errorf("Expected directory error, got: %s", stderr)
	}
}

// Expand tests

func TestExpand(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	testDir := env.CreateTestDir("expandtest")
	env.MustRun("-r", "exp", testDir)

	// Expand should output just the path
	stdout := env.MustRun("-x", "exp")
	if stdout != testDir {
		t.Errorf("Expected %s, got %s", testDir, stdout)
	}
}

func TestExpandNotFound(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	_, stderr, exitCode := env.Run("-x", "nonexistent")
	if exitCode != 1 {
		t.Errorf("Expected exit code 1, got %d", exitCode)
	}
	if !strings.Contains(stderr, "not found") {
		t.Errorf("Expected 'not found' error, got: %s", stderr)
	}
}

// List aliases test (for shell completion)

func TestListAliases(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	// Create multiple aliases
	env.MustRun("-r", "alpha", env.CreateTestDir("a"))
	env.MustRun("-r", "beta", env.CreateTestDir("b"))
	env.MustRun("-r", "gamma", env.CreateTestDir("c"))

	// List aliases (hidden option for completion)
	stdout := env.MustRun("--list-aliases")
	lines := strings.Split(stdout, "\n")

	if len(lines) != 3 {
		t.Errorf("Expected 3 aliases, got %d: %v", len(lines), lines)
	}

	// Should be just names, no paths
	for _, line := range lines {
		if strings.Contains(line, "/") {
			t.Errorf("Expected just alias name, got: %s", line)
		}
	}
}

// Push/Pop tests

func TestPushPop(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	startDir := env.CreateTestDir("start")
	destDir := env.CreateTestDir("destination")

	env.MustRun("-r", "dest", destDir)

	// Simulate being in startDir
	origDir, _ := os.Getwd()
	os.Chdir(startDir)
	defer os.Chdir(origDir)

	// Push should save current dir and output destination
	stdout := env.MustRun("-p", "dest")
	if stdout != destDir {
		t.Errorf("Push: expected %s, got %s", destDir, stdout)
	}

	// Pop should return the saved directory
	stdout = env.MustRun("-o")
	if stdout != startDir {
		t.Errorf("Pop: expected %s, got %s", startDir, stdout)
	}
}

func TestPushMultiple(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	dir1 := env.CreateTestDir("dir1")
	dir2 := env.CreateTestDir("dir2")
	dir3 := env.CreateTestDir("dir3")

	env.MustRun("-r", "one", dir1)
	env.MustRun("-r", "two", dir2)
	env.MustRun("-r", "three", dir3)

	origDir, _ := os.Getwd()
	defer os.Chdir(origDir)

	// Push from dir1 -> dir2
	os.Chdir(dir1)
	env.MustRun("-p", "two")

	// Push from dir2 -> dir3
	os.Chdir(dir2)
	env.MustRun("-p", "three")

	// Pop should return dir2 (LIFO)
	stdout := env.MustRun("-o")
	if stdout != dir2 {
		t.Errorf("First pop: expected %s, got %s", dir2, stdout)
	}

	// Pop should return dir1
	stdout = env.MustRun("-o")
	if stdout != dir1 {
		t.Errorf("Second pop: expected %s, got %s", dir1, stdout)
	}
}

func TestPopEmptyStack(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	_, stderr, exitCode := env.Run("-o")
	if exitCode != 1 {
		t.Errorf("Expected exit code 1, got %d", exitCode)
	}
	if !strings.Contains(stderr, "empty") {
		t.Errorf("Expected 'empty' error, got: %s", stderr)
	}
}

func TestPushInvalidAlias(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	_, stderr, exitCode := env.Run("-p", "nonexistent")
	if exitCode != 1 {
		t.Errorf("Expected exit code 1, got %d", exitCode)
	}
	if !strings.Contains(stderr, "not found") {
		t.Errorf("Expected 'not found' error, got: %s", stderr)
	}
}

func TestPushDeletedDirectory(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	testDir := env.CreateTestDir("pushdeleted")
	env.MustRun("-r", "pushgone", testDir)

	// Delete the target directory
	os.RemoveAll(testDir)

	_, stderr, exitCode := env.Run("-p", "pushgone")
	if exitCode != 2 {
		t.Errorf("Expected exit code 2, got %d", exitCode)
	}
	if !strings.Contains(stderr, "does not exist") {
		t.Errorf("Expected directory error, got: %s", stderr)
	}
}

// Cleanup tests

func TestCleanup(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	// Create directories and register aliases
	validDir := env.CreateTestDir("valid")
	toDeleteDir := env.CreateTestDir("todelete")

	env.MustRun("-r", "valid", validDir)
	env.MustRun("-r", "todelete", toDeleteDir)

	// Delete one directory
	os.RemoveAll(toDeleteDir)

	// Run cleanup
	stdout := env.MustRun("-c")
	if !strings.Contains(stdout, "todelete") {
		t.Errorf("Expected 'todelete' in cleanup output, got: %s", stdout)
	}
	if !strings.Contains(stdout, "Removed") {
		t.Errorf("Expected 'Removed' in output, got: %s", stdout)
	}

	// Verify valid alias still exists
	stdout = env.MustRun("-l")
	if !strings.Contains(stdout, "valid") {
		t.Error("Valid alias should still exist after cleanup")
	}
	if strings.Contains(stdout, "todelete") {
		t.Error("Deleted alias should not exist after cleanup")
	}
}

func TestCleanupMultiple(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	// Create mix of valid and invalid
	valid1 := env.CreateTestDir("valid1")
	valid2 := env.CreateTestDir("valid2")
	invalid1 := env.CreateTestDir("invalid1")
	invalid2 := env.CreateTestDir("invalid2")

	env.MustRun("-r", "v1", valid1)
	env.MustRun("-r", "v2", valid2)
	env.MustRun("-r", "i1", invalid1)
	env.MustRun("-r", "i2", invalid2)

	// Delete invalid directories
	os.RemoveAll(invalid1)
	os.RemoveAll(invalid2)

	// Run cleanup
	stdout := env.MustRun("-c")

	// Both invalid should be reported
	if !strings.Contains(stdout, "i1") || !strings.Contains(stdout, "i2") {
		t.Errorf("Expected both invalid aliases in output, got: %s", stdout)
	}

	// Check final state
	stdout = env.MustRun("-l")
	if !strings.Contains(stdout, "v1") || !strings.Contains(stdout, "v2") {
		t.Error("Valid aliases should remain")
	}
	if strings.Contains(stdout, "i1") || strings.Contains(stdout, "i2") {
		t.Error("Invalid aliases should be removed")
	}
}

func TestCleanupNothingToDo(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	// Create only valid aliases
	validDir := env.CreateTestDir("allvalid")
	env.MustRun("-r", "allvalid", validDir)

	// Run cleanup
	stdout := env.MustRun("-c")
	if !strings.Contains(stdout, "Nothing to clean") {
		t.Errorf("Expected 'Nothing to clean' message, got: %s", stdout)
	}
}

func TestCleanupEmptyDatabase(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	// Run cleanup on empty database
	stdout := env.MustRun("-c")
	if !strings.Contains(stdout, "Nothing to clean") {
		t.Errorf("Expected 'Nothing to clean' message, got: %s", stdout)
	}
}

// CLI Options tests

func TestHelp(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	tests := []string{"-h", "--help"}
	for _, flag := range tests {
		stdout, _, exitCode := env.Run(flag)
		if exitCode != 0 {
			t.Errorf("%s: expected exit 0, got %d", flag, exitCode)
		}
		if !strings.Contains(stdout, "Usage:") {
			t.Errorf("%s: expected usage info, got: %s", flag, stdout)
		}
		// Check for register option in help (could be -r or --register)
		if !strings.Contains(stdout, "-r") && !strings.Contains(stdout, "register") {
			t.Errorf("%s: expected register option in help, got: %s", flag, stdout)
		}
	}
}

func TestVersion(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	tests := []string{"-v", "--version"}
	for _, flag := range tests {
		stdout, _, exitCode := env.Run(flag)
		if exitCode != 0 {
			t.Errorf("%s: expected exit 0, got %d", flag, exitCode)
		}
		if !strings.Contains(stdout, "goto") || !strings.Contains(strings.ToLower(stdout), "version") {
			t.Errorf("%s: expected version info, got: %s", flag, stdout)
		}
	}
}

func TestUnknownOption(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	_, stderr, exitCode := env.Run("--invalid-option")
	if exitCode != 1 {
		t.Errorf("Expected exit 1, got %d", exitCode)
	}
	if !strings.Contains(stderr, "Unknown option") && !strings.Contains(stderr, "unknown") {
		t.Errorf("Expected 'Unknown option' error, got: %s", stderr)
	}
}

func TestNoArguments(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	stdout, stderr, exitCode := env.Run()
	if exitCode != 1 {
		t.Errorf("Expected exit 1, got %d", exitCode)
	}
	// Usage might be on stdout or stderr
	combined := stdout + stderr
	if !strings.Contains(combined, "Usage:") && !strings.Contains(combined, "usage") {
		t.Errorf("Expected usage hint, got stdout: %s, stderr: %s", stdout, stderr)
	}
}

func TestMissingRequiredArgs(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	tests := []struct {
		args []string
		desc string
	}{
		{[]string{"-r"}, "register without args"},
		{[]string{"-r", "alias"}, "register without directory"},
		{[]string{"-u"}, "unregister without alias"},
		{[]string{"-x"}, "expand without alias"},
		{[]string{"-p"}, "push without alias"},
	}

	for _, tt := range tests {
		_, stderr, exitCode := env.Run(tt.args...)
		if exitCode != 1 {
			t.Errorf("%s: expected exit 1, got %d", tt.desc, exitCode)
		}
		// Check for usage hint or error message
		if !strings.Contains(stderr, "Usage:") && !strings.Contains(stderr, "requires") && !strings.Contains(stderr, "missing") {
			t.Errorf("%s: expected usage hint or error, got: %s", tt.desc, stderr)
		}
	}
}

// Edge case tests

func TestPathsWithSpaces(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	spaceDir := env.CreateTestDir("path with spaces")
	env.MustRun("-r", "spaces", spaceDir)

	// Navigate should handle spaces
	stdout := env.MustRun("spaces")
	if stdout != spaceDir {
		t.Errorf("Expected %s, got %s", spaceDir, stdout)
	}

	// List should show full path
	stdout = env.MustRun("-l")
	if !strings.Contains(stdout, "path with spaces") {
		t.Errorf("Expected path with spaces in list, got: %s", stdout)
	}
}

func TestSpecialCharactersInPath(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	// Test various special characters in directory names
	specialDirs := []string{
		"dir-with-hyphens",
		"dir_with_underscores",
		"dir.with.dots",
		"dir'with'quotes",
	}

	for i, name := range specialDirs {
		dir := env.CreateTestDir(name)
		alias := fmt.Sprintf("special%d", i)
		env.MustRun("-r", alias, dir)

		stdout := env.MustRun(alias)
		if stdout != dir {
			t.Errorf("%s: expected %s, got %s", name, dir, stdout)
		}
	}
}

func TestEmptyList(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	stdout := env.MustRun("-l")
	if !strings.Contains(stdout, "No aliases") && !strings.Contains(stdout, "empty") && stdout != "" {
		t.Errorf("Expected 'No aliases' message or empty output, got: %s", stdout)
	}
}

func TestHomeExpansion(t *testing.T) {
	env := Setup(t)
	defer env.Cleanup()

	// This test assumes home directory exists
	home, err := os.UserHomeDir()
	if err != nil {
		t.Skip("Cannot get home directory")
	}

	env.MustRun("-r", "home", "~")

	stdout := env.MustRun("-x", "home")
	if stdout != home {
		t.Errorf("Expected %s, got %s", home, stdout)
	}
}
