package config

import (
	"os"
	"path/filepath"
	"testing"
)

func TestExpandPath(t *testing.T) {
	home, _ := os.UserHomeDir()

	tests := []struct {
		input    string
		expected string
	}{
		{"~", home},
		{"~/test", filepath.Join(home, "test")},
		{".", mustGetwd()},
	}

	for _, tt := range tests {
		got, err := ExpandPath(tt.input)
		if err != nil {
			t.Errorf("ExpandPath(%q): %v", tt.input, err)
			continue
		}
		if got != tt.expected {
			t.Errorf("ExpandPath(%q) = %q, want %q", tt.input, got, tt.expected)
		}
	}
}

func mustGetwd() string {
	wd, _ := os.Getwd()
	return wd
}

func TestDefaultUserConfig(t *testing.T) {
	cfg := DefaultUserConfig()

	if cfg.General.FuzzyThreshold != 0.3 {
		t.Errorf("FuzzyThreshold = %v, want 0.3", cfg.General.FuzzyThreshold)
	}
	if cfg.General.DefaultSort != "alpha" {
		t.Errorf("DefaultSort = %q, want %q", cfg.General.DefaultSort, "alpha")
	}
	if cfg.Display.ShowStats != false {
		t.Errorf("ShowStats = %v, want false", cfg.Display.ShowStats)
	}
	if cfg.Display.ShowTags != true {
		t.Errorf("ShowTags = %v, want true", cfg.Display.ShowTags)
	}
}

func TestLoadUserConfig(t *testing.T) {
	// Create temp directory
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")

	// Write test config
	testConfig := `[general]
fuzzy_threshold = 0.8
default_sort = "usage"

[display]
show_stats = true
show_tags = false
`
	if err := os.WriteFile(configPath, []byte(testConfig), 0644); err != nil {
		t.Fatalf("Failed to write test config: %v", err)
	}

	// Create Config and load
	cfg := &Config{
		ConfigPath: configPath,
		User:       DefaultUserConfig(),
	}

	if err := cfg.loadUserConfig(); err != nil {
		t.Fatalf("loadUserConfig failed: %v", err)
	}

	if cfg.User.General.FuzzyThreshold != 0.8 {
		t.Errorf("FuzzyThreshold = %v, want 0.8", cfg.User.General.FuzzyThreshold)
	}
	if cfg.User.General.DefaultSort != "usage" {
		t.Errorf("DefaultSort = %q, want %q", cfg.User.General.DefaultSort, "usage")
	}
	if cfg.User.Display.ShowStats != true {
		t.Errorf("ShowStats = %v, want true", cfg.User.Display.ShowStats)
	}
	if cfg.User.Display.ShowTags != false {
		t.Errorf("ShowTags = %v, want false", cfg.User.Display.ShowTags)
	}
}

func TestLoadUserConfigPartial(t *testing.T) {
	// Create temp directory
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")

	// Write partial config - only general section
	testConfig := `[general]
fuzzy_threshold = 0.9
`
	if err := os.WriteFile(configPath, []byte(testConfig), 0644); err != nil {
		t.Fatalf("Failed to write test config: %v", err)
	}

	cfg := &Config{
		ConfigPath: configPath,
		User:       DefaultUserConfig(),
	}

	if err := cfg.loadUserConfig(); err != nil {
		t.Fatalf("loadUserConfig failed: %v", err)
	}

	// Changed value
	if cfg.User.General.FuzzyThreshold != 0.9 {
		t.Errorf("FuzzyThreshold = %v, want 0.9", cfg.User.General.FuzzyThreshold)
	}

	// Default values should be preserved
	if cfg.User.General.DefaultSort != "alpha" {
		t.Errorf("DefaultSort = %q, want %q (default)", cfg.User.General.DefaultSort, "alpha")
	}
	if cfg.User.Display.ShowTags != true {
		t.Errorf("ShowTags = %v, want true (default)", cfg.User.Display.ShowTags)
	}
}

func TestLoadUserConfigMissing(t *testing.T) {
	cfg := &Config{
		ConfigPath: "/nonexistent/path/config.toml",
		User:       DefaultUserConfig(),
	}

	err := cfg.loadUserConfig()
	if err == nil {
		t.Error("Expected error for missing config file")
	}
	if !os.IsNotExist(err) {
		t.Errorf("Expected IsNotExist error, got: %v", err)
	}
}

func TestCreateDefaultConfigFile(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")

	cfg := &Config{
		DatabasePath: filepath.Join(tmpDir, "db"),
		ConfigPath:   configPath,
		User:         DefaultUserConfig(),
	}

	// Create default config file
	if err := cfg.CreateDefaultConfigFile(); err != nil {
		t.Fatalf("CreateDefaultConfigFile failed: %v", err)
	}

	// Verify file exists
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		t.Error("Config file was not created")
	}

	// Verify content is loadable
	if err := cfg.loadUserConfig(); err != nil {
		t.Errorf("Failed to load created config: %v", err)
	}

	// Calling again should not error (file exists)
	if err := cfg.CreateDefaultConfigFile(); err != nil {
		t.Errorf("CreateDefaultConfigFile failed on existing file: %v", err)
	}
}

func TestFormatConfig(t *testing.T) {
	cfg := &Config{
		ConfigPath: "/test/path/config.toml",
		User:       DefaultUserConfig(),
	}

	output := cfg.FormatConfig()

	// Check that output contains expected values
	if !contains(output, "/test/path/config.toml") {
		t.Error("FormatConfig missing config path")
	}
	if !contains(output, "0.3") {
		t.Error("FormatConfig missing fuzzy_threshold")
	}
	if !contains(output, "alpha") {
		t.Error("FormatConfig missing default_sort")
	}
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > 0 && containsHelper(s, substr))
}

func containsHelper(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}

func TestConfigMerge(t *testing.T) {
	// Test that partial config file merges with defaults correctly
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")

	// Write partial config - only fuzzy_threshold
	testConfig := `[general]
fuzzy_threshold = 0.5
`
	if err := os.WriteFile(configPath, []byte(testConfig), 0644); err != nil {
		t.Fatalf("Failed to write test config: %v", err)
	}

	cfg := &Config{
		ConfigPath: configPath,
		User:       DefaultUserConfig(),
	}

	if err := cfg.loadUserConfig(); err != nil {
		t.Fatalf("loadUserConfig failed: %v", err)
	}

	// Verify changed value
	if cfg.User.General.FuzzyThreshold != 0.5 {
		t.Errorf("FuzzyThreshold = %v, want 0.5", cfg.User.General.FuzzyThreshold)
	}

	// Verify all defaults are preserved
	if cfg.User.General.DefaultSort != "alpha" {
		t.Errorf("DefaultSort = %q, want 'alpha' (default)", cfg.User.General.DefaultSort)
	}
	if cfg.User.Display.ShowStats != false {
		t.Errorf("ShowStats = %v, want false (default)", cfg.User.Display.ShowStats)
	}
	if cfg.User.Display.ShowTags != true {
		t.Errorf("ShowTags = %v, want true (default)", cfg.User.Display.ShowTags)
	}
}

func TestConfigMergeDisplayOnly(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")

	// Write partial config - only display section
	testConfig := `[display]
show_stats = true
`
	if err := os.WriteFile(configPath, []byte(testConfig), 0644); err != nil {
		t.Fatalf("Failed to write test config: %v", err)
	}

	cfg := &Config{
		ConfigPath: configPath,
		User:       DefaultUserConfig(),
	}

	if err := cfg.loadUserConfig(); err != nil {
		t.Fatalf("loadUserConfig failed: %v", err)
	}

	// Verify changed value
	if cfg.User.Display.ShowStats != true {
		t.Errorf("ShowStats = %v, want true", cfg.User.Display.ShowStats)
	}

	// Verify general defaults preserved
	if cfg.User.General.FuzzyThreshold != 0.3 {
		t.Errorf("FuzzyThreshold = %v, want 0.3 (default)", cfg.User.General.FuzzyThreshold)
	}
	if cfg.User.General.DefaultSort != "alpha" {
		t.Errorf("DefaultSort = %q, want 'alpha' (default)", cfg.User.General.DefaultSort)
	}
}

func TestEnsureConfigDir(t *testing.T) {
	tmpDir := t.TempDir()
	nestedPath := filepath.Join(tmpDir, "nested", "deep", "config")

	cfg := &Config{
		DatabasePath: nestedPath,
		ConfigPath:   filepath.Join(nestedPath, "config.toml"),
	}

	if err := cfg.EnsureConfigDir(); err != nil {
		t.Fatalf("EnsureConfigDir failed: %v", err)
	}

	// Verify directory was created
	parentDir := filepath.Dir(nestedPath)
	info, err := os.Stat(parentDir)
	if os.IsNotExist(err) {
		t.Errorf("Config directory was not created: %s", parentDir)
	}
	if err == nil && !info.IsDir() {
		t.Errorf("Config path is not a directory: %s", parentDir)
	}
}

func TestExpandPathEnvVar(t *testing.T) {
	// Set a test environment variable
	testValue := "/test/path"
	os.Setenv("GOTO_TEST_PATH", testValue)
	defer os.Unsetenv("GOTO_TEST_PATH")

	result, err := ExpandPath("$GOTO_TEST_PATH/subdir")
	if err != nil {
		t.Fatalf("ExpandPath failed: %v", err)
	}

	expected := filepath.Join(testValue, "subdir")
	if result != expected {
		t.Errorf("ExpandPath('$GOTO_TEST_PATH/subdir') = %q, want %q", result, expected)
	}
}

func TestExpandPathDotDot(t *testing.T) {
	cwd, _ := os.Getwd()
	parent := filepath.Dir(cwd)

	result, err := ExpandPath("..")
	if err != nil {
		t.Fatalf("ExpandPath failed: %v", err)
	}

	if result != parent {
		t.Errorf("ExpandPath('..') = %q, want %q", result, parent)
	}
}

func TestExpandPathRelative(t *testing.T) {
	cwd, _ := os.Getwd()

	result, err := ExpandPath("./subdir")
	if err != nil {
		t.Fatalf("ExpandPath failed: %v", err)
	}

	expected := filepath.Join(cwd, "subdir")
	if result != expected {
		t.Errorf("ExpandPath('./subdir') = %q, want %q", result, expected)
	}
}

func TestExpandPathAbsolute(t *testing.T) {
	absolutePath := "/absolute/path"

	result, err := ExpandPath(absolutePath)
	if err != nil {
		t.Fatalf("ExpandPath failed: %v", err)
	}

	if result != absolutePath {
		t.Errorf("ExpandPath(%q) = %q, want %q", absolutePath, result, absolutePath)
	}
}

func TestConfigInvalidTOML(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")

	// Write invalid TOML
	invalidConfig := `[general
fuzzy_threshold = 0.5`
	if err := os.WriteFile(configPath, []byte(invalidConfig), 0644); err != nil {
		t.Fatalf("Failed to write test config: %v", err)
	}

	cfg := &Config{
		ConfigPath: configPath,
		User:       DefaultUserConfig(),
	}

	err := cfg.loadUserConfig()
	if err == nil {
		t.Error("Expected error for invalid TOML, got nil")
	}
	if !contains(err.Error(), "parsing config file") {
		t.Errorf("Expected 'parsing config file' error, got: %v", err)
	}
}

func TestLoadWithGOTO_DB(t *testing.T) {
	tmpDir := t.TempDir()
	testPath := filepath.Join(tmpDir, "custom_db")

	// Set environment variable
	oldDB := os.Getenv("GOTO_DB")
	os.Setenv("GOTO_DB", testPath)
	defer func() {
		if oldDB == "" {
			os.Unsetenv("GOTO_DB")
		} else {
			os.Setenv("GOTO_DB", oldDB)
		}
	}()

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load failed: %v", err)
	}

	if cfg.DatabasePath != testPath {
		t.Errorf("DatabasePath = %q, want %q", cfg.DatabasePath, testPath)
	}
}

func TestLoadWithXDG_CONFIG_HOME(t *testing.T) {
	tmpDir := t.TempDir()

	// Clear GOTO_DB and set XDG_CONFIG_HOME
	oldDB := os.Getenv("GOTO_DB")
	oldXDG := os.Getenv("XDG_CONFIG_HOME")
	os.Unsetenv("GOTO_DB")
	os.Setenv("XDG_CONFIG_HOME", tmpDir)
	defer func() {
		if oldDB == "" {
			os.Unsetenv("GOTO_DB")
		} else {
			os.Setenv("GOTO_DB", oldDB)
		}
		if oldXDG == "" {
			os.Unsetenv("XDG_CONFIG_HOME")
		} else {
			os.Setenv("XDG_CONFIG_HOME", oldXDG)
		}
	}()

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load failed: %v", err)
	}

	expected := filepath.Join(tmpDir, "goto")
	if cfg.DatabasePath != expected {
		t.Errorf("DatabasePath = %q, want %q", cfg.DatabasePath, expected)
	}
}

func TestConfigStackPath(t *testing.T) {
	tmpDir := t.TempDir()
	testPath := filepath.Join(tmpDir, "db")

	oldDB := os.Getenv("GOTO_DB")
	os.Setenv("GOTO_DB", testPath)
	defer func() {
		if oldDB == "" {
			os.Unsetenv("GOTO_DB")
		} else {
			os.Setenv("GOTO_DB", oldDB)
		}
	}()

	cfg, err := Load()
	if err != nil {
		t.Fatalf("Load failed: %v", err)
	}

	expectedStack := filepath.Join(tmpDir, "goto_stack")
	if cfg.StackPath != expectedStack {
		t.Errorf("StackPath = %q, want %q", cfg.StackPath, expectedStack)
	}
}

func TestCreateDefaultConfigFileExisting(t *testing.T) {
	tmpDir := t.TempDir()
	configPath := filepath.Join(tmpDir, "config.toml")

	// Write existing config with custom value
	existingConfig := `[general]
fuzzy_threshold = 0.99
`
	if err := os.WriteFile(configPath, []byte(existingConfig), 0644); err != nil {
		t.Fatalf("Failed to write existing config: %v", err)
	}

	cfg := &Config{
		DatabasePath: filepath.Join(tmpDir, "db"),
		ConfigPath:   configPath,
		User:         DefaultUserConfig(),
	}

	// CreateDefaultConfigFile should not overwrite
	if err := cfg.CreateDefaultConfigFile(); err != nil {
		t.Fatalf("CreateDefaultConfigFile failed: %v", err)
	}

	// Verify existing config was not overwritten
	if err := cfg.loadUserConfig(); err != nil {
		t.Fatalf("loadUserConfig failed: %v", err)
	}

	if cfg.User.General.FuzzyThreshold != 0.99 {
		t.Errorf("FuzzyThreshold = %v, want 0.99 (existing value should be preserved)", cfg.User.General.FuzzyThreshold)
	}
}

func TestDefaultUserConfigValues(t *testing.T) {
	cfg := DefaultUserConfig()

	// Check all default values explicitly
	tests := []struct {
		name     string
		got      interface{}
		expected interface{}
	}{
		{"FuzzyThreshold", cfg.General.FuzzyThreshold, 0.3},
		{"DefaultSort", cfg.General.DefaultSort, "alpha"},
		{"ShowStats", cfg.Display.ShowStats, false},
		{"ShowTags", cfg.Display.ShowTags, true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.got != tt.expected {
				t.Errorf("%s = %v, want %v", tt.name, tt.got, tt.expected)
			}
		})
	}
}
