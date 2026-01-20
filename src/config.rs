//! Configuration loading and path handling

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to determine home directory")]
    NoHomeDir,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_fuzzy_threshold")]
    pub fuzzy_threshold: f64,

    #[serde(default = "default_sort")]
    pub default_sort: String,
}

fn default_fuzzy_threshold() -> f64 {
    0.3
}

fn default_sort() -> String {
    "alpha".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            fuzzy_threshold: default_fuzzy_threshold(),
            default_sort: default_sort(),
        }
    }
}

/// Display settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default)]
    pub show_stats: bool,

    #[serde(default = "default_show_tags")]
    pub show_tags: bool,
}

fn default_show_tags() -> bool {
    true
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_stats: false,
            show_tags: true,
        }
    }
}

/// User-configurable settings loaded from TOML
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub display: DisplayConfig,
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the database directory (contains aliases, config, stack)
    pub database_path: PathBuf,
    /// Path to the directory stack file
    pub stack_path: PathBuf,
    /// Path to the config.toml file
    pub config_path: PathBuf,
    /// Path to the aliases database file
    pub aliases_path: PathBuf,
    /// User configuration loaded from config.toml
    pub user: UserConfig,
}

impl Config {
    /// Load configuration from environment and defaults
    pub fn load() -> Result<Self, ConfigError> {
        let base_path = get_database_path()?;

        let config_path = base_path.join("config.toml");
        let stack_path = base_path.join("goto_stack");
        let aliases_path = base_path.join("aliases.toml");

        let user = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            UserConfig::default()
        };

        Ok(Config {
            database_path: base_path,
            stack_path,
            config_path,
            aliases_path,
            user,
        })
    }

    /// Ensure the config directory exists
    pub fn ensure_dirs(&self) -> Result<(), ConfigError> {
        fs::create_dir_all(&self.database_path)?;
        Ok(())
    }

    /// Create the default config file if it doesn't exist
    pub fn create_default_config_file(&self) -> Result<(), ConfigError> {
        if self.config_path.exists() {
            return Ok(());
        }

        self.ensure_dirs()?;

        let default_config = r#"[general]
fuzzy_threshold = 0.6
default_sort = "alpha"  # alpha, usage, recent

[display]
show_stats = false
show_tags = true
"#;

        fs::write(&self.config_path, default_config)?;
        Ok(())
    }

    /// Format the current configuration as a string
    pub fn format_config(&self) -> String {
        format!(
            "Configuration file: {}\n\n\
             [general]\n\
             fuzzy_threshold = {:.1}\n\
             default_sort = \"{}\"\n\n\
             [display]\n\
             show_stats = {}\n\
             show_tags = {}\n",
            self.config_path.display(),
            self.user.general.fuzzy_threshold,
            self.user.general.default_sort,
            self.user.display.show_stats,
            self.user.display.show_tags,
        )
    }
}

/// Get the database path based on priority:
/// 1. $GOTO_DB environment variable
/// 2. $XDG_CONFIG_HOME/goto
/// 3. ~/.config/goto
fn get_database_path() -> Result<PathBuf, ConfigError> {
    // Check GOTO_DB env var first
    if let Ok(path) = std::env::var("GOTO_DB") {
        return Ok(PathBuf::from(path));
    }

    // Check XDG_CONFIG_HOME
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("goto"));
    }

    // Default to ~/.config/goto
    dirs::home_dir()
        .map(|h| h.join(".config").join("goto"))
        .ok_or(ConfigError::NoHomeDir)
}

/// Expand ~, environment variables, and convert to absolute path
pub fn expand_path(path: &str) -> Result<PathBuf, ConfigError> {
    let expanded = if path.starts_with('~') {
        let home = dirs::home_dir().ok_or(ConfigError::NoHomeDir)?;
        let rest = path[1..].trim_start_matches('/');
        if rest.is_empty() {
            home
        } else {
            home.join(rest)
        }
    } else {
        PathBuf::from(shellexpand::env(path).unwrap_or(path.into()).into_owned())
    };

    // Try to canonicalize, but fall back to the expanded path if it doesn't exist
    Ok(std::fs::canonicalize(&expanded).unwrap_or(expanded))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Mutex to ensure environment-modifying tests run serially
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// Helper to run a test that modifies environment variables safely
    fn with_env_vars<F, R>(vars: &[(&str, Option<&str>)], test_fn: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = ENV_MUTEX.lock().unwrap();

        // Save original values
        let originals: Vec<_> = vars
            .iter()
            .map(|(name, _)| (*name, env::var(name).ok()))
            .collect();

        // Set new values
        for (name, value) in vars {
            match value {
                Some(v) => env::set_var(name, v),
                None => env::remove_var(name),
            }
        }

        let result = test_fn();

        // Restore original values
        for (name, original) in originals {
            match original {
                Some(v) => env::set_var(name, v),
                None => env::remove_var(name),
            }
        }

        result
    }

    #[test]
    fn test_config_load() {
        with_env_vars(&[], || {
            let config = Config::load();
            assert!(config.is_ok());
        });
    }

    #[test]
    fn test_config_paths() {
        with_env_vars(&[], || {
            let config = Config::load().unwrap();
            assert!(config.aliases_path.to_string_lossy().contains("aliases.toml"));
            assert!(config.stack_path.to_string_lossy().contains("goto_stack"));
        });
    }

    #[test]
    fn test_default_user_config() {
        let user = UserConfig::default();
        assert!((user.general.fuzzy_threshold - 0.3).abs() < f64::EPSILON);
        assert_eq!(user.general.default_sort, "alpha");
        assert!(!user.display.show_stats);
        assert!(user.display.show_tags);
    }

    #[test]
    fn test_expand_path_tilde() {
        let home = dirs::home_dir().unwrap();
        let expanded = expand_path("~").unwrap();
        assert_eq!(expanded, home);

        let expanded = expand_path("~/test").unwrap();
        assert_eq!(expanded, home.join("test"));
    }

    #[test]
    fn test_expand_path_env_var() {
        with_env_vars(&[("TEST_EXPAND_VAR", Some("/tmp/test"))], || {
            let expanded = expand_path("$TEST_EXPAND_VAR/subdir").unwrap();
            assert!(expanded.to_string_lossy().contains("/tmp/test/subdir"));
        });
    }

    #[test]
    fn test_parse_user_config() {
        let toml_str = r#"
[general]
fuzzy_threshold = 0.5
default_sort = "recent"

[display]
show_stats = true
show_tags = false
"#;
        let config: UserConfig = toml::from_str(toml_str).unwrap();
        assert!((config.general.fuzzy_threshold - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.general.default_sort, "recent");
        assert!(config.display.show_stats);
        assert!(!config.display.show_tags);
    }

    #[test]
    fn test_parse_partial_config() {
        // Test that missing fields use defaults
        let toml_str = r#"
[general]
fuzzy_threshold = 0.7
"#;
        let config: UserConfig = toml::from_str(toml_str).unwrap();
        assert!((config.general.fuzzy_threshold - 0.7).abs() < f64::EPSILON);
        assert_eq!(config.general.default_sort, "alpha"); // default
        assert!(!config.display.show_stats); // default
        assert!(config.display.show_tags); // default
    }

    #[test]
    fn test_format_config() {
        // Use a manually constructed config to avoid env var issues
        let temp_dir = tempfile::tempdir().unwrap();
        let config = Config {
            database_path: temp_dir.path().to_path_buf(),
            stack_path: temp_dir.path().join("goto_stack"),
            config_path: temp_dir.path().join("config.toml"),
            aliases_path: temp_dir.path().join("aliases.toml"),
            user: UserConfig::default(),
        };
        let formatted = config.format_config();
        assert!(formatted.contains("Configuration file:"));
        assert!(formatted.contains("fuzzy_threshold"));
        assert!(formatted.contains("default_sort"));
        assert!(formatted.contains("show_stats"));
        assert!(formatted.contains("show_tags"));
    }

    #[test]
    fn test_goto_db_env_var() {
        with_env_vars(&[("GOTO_DB", Some("/custom/path"))], || {
            let path = get_database_path().unwrap();
            assert_eq!(path, PathBuf::from("/custom/path"));
        });
    }

    #[test]
    fn test_xdg_config_home_env_var() {
        with_env_vars(
            &[
                ("GOTO_DB", None),
                ("XDG_CONFIG_HOME", Some("/tmp/test-xdg-config")),
            ],
            || {
                let path = get_database_path().unwrap();
                assert_eq!(path, PathBuf::from("/tmp/test-xdg-config/goto"));
            },
        );
    }

    #[test]
    fn test_config_load_with_existing_config_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write a custom config file
        let custom_config = r#"
[general]
fuzzy_threshold = 0.8
default_sort = "usage"

[display]
show_stats = true
show_tags = false
"#;
        fs::write(&config_path, custom_config).unwrap();

        with_env_vars(
            &[("GOTO_DB", Some(temp_dir.path().to_str().unwrap()))],
            || {
                let config = Config::load().unwrap();

                // Verify the config was loaded from file
                assert!((config.user.general.fuzzy_threshold - 0.8).abs() < f64::EPSILON);
                assert_eq!(config.user.general.default_sort, "usage");
                assert!(config.user.display.show_stats);
                assert!(!config.user.display.show_tags);
            },
        );
    }

    #[test]
    fn test_ensure_dirs_creates_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested").join("config");

        let config = Config {
            database_path: nested_path.clone(),
            stack_path: nested_path.join("goto_stack"),
            config_path: nested_path.join("config.toml"),
            aliases_path: nested_path.join("aliases.toml"),
            user: UserConfig::default(),
        };

        assert!(!nested_path.exists());
        config.ensure_dirs().unwrap();
        assert!(nested_path.exists());
    }

    #[test]
    fn test_create_default_config_file_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            database_path: temp_dir.path().to_path_buf(),
            stack_path: temp_dir.path().join("goto_stack"),
            config_path: config_path.clone(),
            aliases_path: temp_dir.path().join("aliases.toml"),
            user: UserConfig::default(),
        };

        assert!(!config_path.exists());
        config.create_default_config_file().unwrap();
        assert!(config_path.exists());

        // Verify the content
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("fuzzy_threshold"));
        assert!(content.contains("default_sort"));
        assert!(content.contains("show_stats"));
        assert!(content.contains("show_tags"));
    }

    #[test]
    fn test_create_default_config_file_already_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create an existing config file with custom content
        let custom_content = "# custom config";
        fs::write(&config_path, custom_content).unwrap();

        let config = Config {
            database_path: temp_dir.path().to_path_buf(),
            stack_path: temp_dir.path().join("goto_stack"),
            config_path: config_path.clone(),
            aliases_path: temp_dir.path().join("aliases.toml"),
            user: UserConfig::default(),
        };

        // Should return early without overwriting
        config.create_default_config_file().unwrap();

        // Verify original content is preserved
        let content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(content, custom_content);
    }

    #[test]
    fn test_create_default_config_file_creates_dirs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_dir = temp_dir.path().join("nested").join("deeply");
        let config_path = nested_dir.join("config.toml");

        let config = Config {
            database_path: nested_dir.clone(),
            stack_path: nested_dir.join("goto_stack"),
            config_path: config_path.clone(),
            aliases_path: nested_dir.join("aliases.toml"),
            user: UserConfig::default(),
        };

        assert!(!nested_dir.exists());
        config.create_default_config_file().unwrap();
        assert!(nested_dir.exists());
        assert!(config_path.exists());
    }

    #[test]
    fn test_expand_path_missing_env_var() {
        with_env_vars(&[("NONEXISTENT_VAR_FOR_TEST_12345", None)], || {
            // shellexpand leaves unknown env vars in place or returns the original
            let result = expand_path("$NONEXISTENT_VAR_FOR_TEST_12345/foo").unwrap();
            // The path should still be returned (shellexpand handles missing vars gracefully)
            assert!(result.to_string_lossy().contains("foo"));
        });
    }

    #[test]
    fn test_config_load_with_invalid_toml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write invalid TOML
        fs::write(&config_path, "invalid { toml [syntax").unwrap();

        with_env_vars(
            &[("GOTO_DB", Some(temp_dir.path().to_str().unwrap()))],
            || {
                // Should return an error for invalid TOML
                let result = Config::load();
                assert!(result.is_err());
            },
        );
    }

    #[test]
    fn test_config_load_without_config_file() {
        let temp_dir = tempfile::tempdir().unwrap();

        with_env_vars(
            &[("GOTO_DB", Some(temp_dir.path().to_str().unwrap()))],
            || {
                let config = Config::load().unwrap();

                // Should use defaults
                assert!((config.user.general.fuzzy_threshold - 0.3).abs() < f64::EPSILON);
                assert_eq!(config.user.general.default_sort, "alpha");
                assert!(!config.user.display.show_stats);
                assert!(config.user.display.show_tags);
            },
        );
    }
}
