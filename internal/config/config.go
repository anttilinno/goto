package config

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

// GeneralConfig holds general application settings
type GeneralConfig struct {
	FuzzyThreshold float64 `toml:"fuzzy_threshold"`
	DefaultSort    string  `toml:"default_sort"` // alpha, usage, recent
}

// DisplayConfig holds display settings
type DisplayConfig struct {
	ShowStats bool `toml:"show_stats"`
	ShowTags  bool `toml:"show_tags"`
}

// UserConfig holds user-configurable settings loaded from TOML
type UserConfig struct {
	General GeneralConfig `toml:"general"`
	Display DisplayConfig `toml:"display"`
}

// Config holds application configuration
type Config struct {
	DatabasePath string
	StackPath    string
	ConfigPath   string
	User         *UserConfig
}

// DefaultUserConfig returns the default user configuration
func DefaultUserConfig() *UserConfig {
	return &UserConfig{
		General: GeneralConfig{
			FuzzyThreshold: 0.3, // Lower threshold to catch common typos (transpositions)
			DefaultSort:    "alpha",
		},
		Display: DisplayConfig{
			ShowStats: false,
			ShowTags:  true,
		},
	}
}

// Load returns the configuration with resolved paths
func Load() (*Config, error) {
	dbPath, err := getDatabasePath()
	if err != nil {
		return nil, err
	}

	dir := filepath.Dir(dbPath)
	stackPath := filepath.Join(dir, "goto_stack")
	configPath := filepath.Join(dir, "config.toml")

	cfg := &Config{
		DatabasePath: dbPath,
		StackPath:    stackPath,
		ConfigPath:   configPath,
		User:         DefaultUserConfig(),
	}

	// Try to load user config from file
	if err := cfg.loadUserConfig(); err != nil {
		// Only return error if it's not a "file not found" error
		if !os.IsNotExist(err) {
			return nil, err
		}
	}

	return cfg, nil
}

// loadUserConfig loads user configuration from TOML file
func (c *Config) loadUserConfig() error {
	data, err := os.ReadFile(c.ConfigPath)
	if err != nil {
		return err
	}

	// Start with defaults
	userCfg := DefaultUserConfig()

	// Decode TOML over defaults (missing values keep defaults)
	if _, err := toml.Decode(string(data), userCfg); err != nil {
		return fmt.Errorf("parsing config file: %w", err)
	}

	c.User = userCfg
	return nil
}

// CreateDefaultConfigFile creates the config file with default values if it doesn't exist
func (c *Config) CreateDefaultConfigFile() error {
	// Check if file already exists
	if _, err := os.Stat(c.ConfigPath); err == nil {
		return nil // File exists, don't overwrite
	}

	// Ensure directory exists
	if err := c.EnsureConfigDir(); err != nil {
		return err
	}

	defaultConfig := `[general]
fuzzy_threshold = 0.6
default_sort = "alpha"  # alpha, usage, recent

[display]
show_stats = false
show_tags = true
`

	return os.WriteFile(c.ConfigPath, []byte(defaultConfig), 0644)
}

// FormatConfig returns the current configuration as a formatted string
func (c *Config) FormatConfig() string {
	return fmt.Sprintf(`Configuration file: %s

[general]
fuzzy_threshold = %.1f
default_sort = "%s"

[display]
show_stats = %t
show_tags = %t
`, c.ConfigPath, c.User.General.FuzzyThreshold, c.User.General.DefaultSort,
		c.User.Display.ShowStats, c.User.Display.ShowTags)
}

// getDatabasePath returns the database file path based on priority:
// 1. $GOTO_DB environment variable
// 2. $XDG_CONFIG_HOME/goto
// 3. ~/.config/goto
func getDatabasePath() (string, error) {
	// Check GOTO_DB env var first
	if envPath := os.Getenv("GOTO_DB"); envPath != "" {
		return envPath, nil
	}

	// Check XDG_CONFIG_HOME
	if xdgConfig := os.Getenv("XDG_CONFIG_HOME"); xdgConfig != "" {
		return filepath.Join(xdgConfig, "goto"), nil
	}

	// Default to ~/.config/goto
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}

	return filepath.Join(home, ".config", "goto"), nil
}

// EnsureConfigDir creates the config directory if it doesn't exist
func (c *Config) EnsureConfigDir() error {
	dir := filepath.Dir(c.DatabasePath)
	return os.MkdirAll(dir, 0755)
}

// ExpandPath expands ~, ., and environment variables in a path
func ExpandPath(path string) (string, error) {
	// Expand ~ to home directory
	if len(path) > 0 && path[0] == '~' {
		home, err := os.UserHomeDir()
		if err != nil {
			return "", err
		}
		path = filepath.Join(home, path[1:])
	}

	// Expand environment variables
	path = os.ExpandEnv(path)

	// Convert to absolute path (handles . and ..)
	return filepath.Abs(path)
}
