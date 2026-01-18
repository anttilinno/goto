package integration

import (
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// TestEnv holds the test environment
type TestEnv struct {
	T          *testing.T
	TmpDir     string
	BinaryPath string
	ConfigDir  string
	DBPath     string
	StackPath  string
}

// Setup creates a fresh test environment
func Setup(t *testing.T) *TestEnv {
	t.Helper()

	// Find binary (assume it's built in project root)
	binaryPath := findBinary(t)

	// Create temp directory for this test
	tmpDir, err := os.MkdirTemp("", "goto-integration-*")
	if err != nil {
		t.Fatalf("Failed to create temp dir: %v", err)
	}

	configDir := filepath.Join(tmpDir, "config")
	if err := os.MkdirAll(configDir, 0755); err != nil {
		t.Fatalf("Failed to create config dir: %v", err)
	}

	env := &TestEnv{
		T:          t,
		TmpDir:     tmpDir,
		BinaryPath: binaryPath,
		ConfigDir:  configDir,
		DBPath:     filepath.Join(configDir, "goto"),
		StackPath:  filepath.Join(configDir, "goto_stack"),
	}

	return env
}

// Cleanup removes the test environment
func (e *TestEnv) Cleanup() {
	os.RemoveAll(e.TmpDir)
}

// Run executes goto-bin with args and returns stdout, stderr, exit code
func (e *TestEnv) Run(args ...string) (stdout, stderr string, exitCode int) {
	cmd := exec.Command(e.BinaryPath, args...)
	cmd.Env = append(os.Environ(), "GOTO_DB="+e.DBPath)

	var outBuf, errBuf strings.Builder
	cmd.Stdout = &outBuf
	cmd.Stderr = &errBuf

	err := cmd.Run()
	exitCode = 0
	if exitErr, ok := err.(*exec.ExitError); ok {
		exitCode = exitErr.ExitCode()
	} else if err != nil {
		e.T.Fatalf("Failed to run command: %v", err)
	}

	return strings.TrimSpace(outBuf.String()), strings.TrimSpace(errBuf.String()), exitCode
}

// MustRun executes and fails test if exit code != 0
func (e *TestEnv) MustRun(args ...string) string {
	stdout, stderr, exitCode := e.Run(args...)
	if exitCode != 0 {
		e.T.Fatalf("Command failed: goto %v\nstdout: %s\nstderr: %s\nexit: %d",
			args, stdout, stderr, exitCode)
	}
	return stdout
}

// CreateTestDir creates a directory in the temp folder
func (e *TestEnv) CreateTestDir(name string) string {
	dir := filepath.Join(e.TmpDir, name)
	if err := os.MkdirAll(dir, 0755); err != nil {
		e.T.Fatalf("Failed to create test dir: %v", err)
	}
	return dir
}

func findBinary(t *testing.T) string {
	// Look for binary relative to test location
	candidates := []string{
		"../../goto-bin",
		"../goto-bin",
		"./goto-bin",
	}
	for _, c := range candidates {
		if _, err := os.Stat(c); err == nil {
			abs, _ := filepath.Abs(c)
			return abs
		}
	}
	t.Fatal("goto-bin not found. Run 'mise run build' first.")
	return ""
}
