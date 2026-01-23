//! Prune notification for stale aliases
//!
//! Alerts users when aliases point to missing directories, prompting cleanup.
//! Rate-limited checks and snooze capability to avoid notification spam.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::database::Database;

/// Cached prune check state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruneCache {
    /// Last time we checked for stale aliases
    pub last_check: DateTime<Utc>,
    /// Number of stale aliases found at last check
    pub stale_count: usize,
    /// If set, notifications are snoozed until this time
    pub snoozed_until: Option<DateTime<Utc>>,
}

impl Default for PruneCache {
    fn default() -> Self {
        Self {
            last_check: DateTime::from_timestamp(0, 0).unwrap(),
            stale_count: 0,
            snoozed_until: None,
        }
    }
}

/// Get the path to the prune cache file
fn cache_path(config: &Config) -> PathBuf {
    config.database_path.join("prune_cache.json")
}

/// Load the prune cache from disk
fn load_cache(config: &Config) -> PruneCache {
    let path = cache_path(config);
    if !path.exists() {
        return PruneCache::default();
    }

    match File::open(&path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_default()
        }
        Err(_) => PruneCache::default(),
    }
}

/// Save the prune cache to disk
fn save_cache(config: &Config, cache: &PruneCache) -> Result<(), Box<dyn Error>> {
    config.ensure_dirs()?;
    let path = cache_path(config);
    let file = File::create(&path)?;
    serde_json::to_writer_pretty(file, cache)?;
    Ok(())
}

/// Count aliases pointing to non-existent directories
pub fn count_stale_aliases(db: &Database) -> usize {
    db.all()
        .filter(|a| !Path::new(&a.path).exists())
        .count()
}

/// Check for stale aliases (respects rate limit)
///
/// Returns number of stale aliases, or None if check was skipped due to rate limit
pub fn check_for_stale_aliases(db: &Database, config: &Config) -> Option<usize> {
    let mut cache = load_cache(config);
    let check_interval = Duration::hours(config.user.prune.check_interval_hours as i64);

    // Skip if checked recently - return cached count
    if Utc::now() - cache.last_check < check_interval {
        return Some(cache.stale_count);
    }

    // Perform the check
    let stale_count = count_stale_aliases(db);

    // Update cache
    cache.last_check = Utc::now();
    cache.stale_count = stale_count;
    let _ = save_cache(config, &cache); // Best-effort save

    Some(stale_count)
}

/// Show stale alias notification if appropriate
///
/// Should be called after list/stats/tags commands complete.
/// Does NOT add latency - uses cached data when possible.
pub fn notify_if_stale_aliases(config: &Config, db: &Database) {
    if !config.user.prune.auto_check {
        return;
    }

    let cache = load_cache(config);

    // Check if snoozed
    if let Some(snoozed_until) = cache.snoozed_until {
        if Utc::now() < snoozed_until {
            return; // Still snoozed
        }
    }

    // Check if we need to perform a fresh check
    let check_interval = Duration::hours(config.user.prune.check_interval_hours as i64);
    if Utc::now() - cache.last_check >= check_interval {
        // Perform check and update cache - don't show notification on same invocation
        let _ = check_for_stale_aliases(db, config);
        return;
    }

    // Show notification if stale aliases exist
    if cache.stale_count > 0 {
        eprintln!(
            "Note: {} alias{} point to missing directories. Run 'goto --cleanup' to review.",
            cache.stale_count,
            if cache.stale_count == 1 { "" } else { "es" }
        );
    }
}

/// Snooze prune notifications for the specified number of days
pub fn snooze_notifications(config: &Config, days: u32) -> Result<(), Box<dyn Error>> {
    let mut cache = load_cache(config);
    cache.snoozed_until = Some(Utc::now() + Duration::days(days as i64));
    save_cache(config, &cache)?;
    println!("Prune notifications snoozed for {} days.", days);
    Ok(())
}

/// Reset the prune cache (called after cleanup)
///
/// Clears the stale count so notification doesn't appear until next check.
pub fn reset_cache(config: &Config) -> Result<(), Box<dyn Error>> {
    let mut cache = load_cache(config);
    cache.stale_count = 0;
    save_cache(config, &cache)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::Alias;
    use crate::config::UserConfig;
    use std::fs;
    use tempfile::TempDir;

    /// Create a test config with a temp directory
    fn test_config(temp_dir: &std::path::Path) -> Config {
        Config {
            database_path: temp_dir.to_path_buf(),
            stack_path: temp_dir.join("goto_stack"),
            config_path: temp_dir.join("config.toml"),
            aliases_path: temp_dir.join("aliases.toml"),
            user: UserConfig::default(),
        }
    }

    #[test]
    fn test_prune_cache_default() {
        let cache = PruneCache::default();
        assert_eq!(cache.stale_count, 0);
        assert!(cache.snoozed_until.is_none());
        // last_check should be at epoch 0
        assert_eq!(cache.last_check.timestamp(), 0);
    }

    #[test]
    fn test_prune_cache_serialization() {
        let cache = PruneCache {
            last_check: Utc::now(),
            stale_count: 5,
            snoozed_until: Some(Utc::now() + Duration::days(7)),
        };

        let json = serde_json::to_string(&cache).unwrap();
        let loaded: PruneCache = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.stale_count, 5);
        assert!(loaded.snoozed_until.is_some());
    }

    #[test]
    fn test_load_cache_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        let cache = load_cache(&config);
        assert_eq!(cache.stale_count, 0);
        assert!(cache.snoozed_until.is_none());
    }

    #[test]
    fn test_load_cache_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Write invalid JSON
        let cache_file = temp_dir.path().join("prune_cache.json");
        fs::write(&cache_file, "not valid json").unwrap();

        // Should return default cache on parse error
        let cache = load_cache(&config);
        assert_eq!(cache.stale_count, 0);
    }

    #[test]
    fn test_save_and_load_cache() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        let cache = PruneCache {
            last_check: Utc::now(),
            stale_count: 3,
            snoozed_until: None,
        };

        save_cache(&config, &cache).unwrap();

        let loaded = load_cache(&config);
        assert_eq!(loaded.stale_count, 3);
    }

    #[test]
    fn test_count_stale_aliases() {
        let temp_dir = TempDir::new().unwrap();
        let db_file = temp_dir.path().join("aliases.toml");
        let mut db = crate::database::Database::load_from_path(&db_file).unwrap();

        // Add valid alias (temp_dir exists)
        db.insert(Alias::new("valid", temp_dir.path().to_str().unwrap()).unwrap());

        // Add invalid alias (path doesn't exist)
        db.insert(Alias::new("invalid", "/nonexistent/path/12345").unwrap());

        let count = count_stale_aliases(&db);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_snooze_sets_snoozed_until() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        snooze_notifications(&config, 7).unwrap();

        let cache = load_cache(&config);
        assert!(cache.snoozed_until.is_some());

        let snoozed_until = cache.snoozed_until.unwrap();
        let now = Utc::now();
        // Should be approximately 7 days in the future
        let diff = snoozed_until - now;
        assert!(diff.num_days() >= 6 && diff.num_days() <= 7);
    }

    #[test]
    fn test_reset_cache() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Save cache with stale count
        let cache = PruneCache {
            last_check: Utc::now(),
            stale_count: 5,
            snoozed_until: None,
        };
        save_cache(&config, &cache).unwrap();

        // Reset cache
        reset_cache(&config).unwrap();

        // Verify stale count is now 0
        let loaded = load_cache(&config);
        assert_eq!(loaded.stale_count, 0);
    }

    #[test]
    fn test_cache_path() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());
        let path = cache_path(&config);
        assert_eq!(path, temp_dir.path().join("prune_cache.json"));
    }

    #[test]
    fn test_check_for_stale_aliases_returns_count() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());
        let db_file = temp_dir.path().join("aliases.toml");
        let mut db = crate::database::Database::load_from_path(&db_file).unwrap();

        // Add an invalid alias
        db.insert(Alias::new("invalid", "/nonexistent/path/67890").unwrap());

        let count = check_for_stale_aliases(&db, &config);
        assert_eq!(count, Some(1));

        // Second call should return cached count without re-checking
        let cached_count = check_for_stale_aliases(&db, &config);
        assert_eq!(cached_count, Some(1));
    }

    #[test]
    fn test_notify_disabled_when_auto_check_false() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = test_config(temp_dir.path());
        config.user.prune.auto_check = false;

        let db_file = temp_dir.path().join("aliases.toml");
        let db = crate::database::Database::load_from_path(&db_file).unwrap();

        // Should return early without any errors
        notify_if_stale_aliases(&config, &db);
    }

    #[test]
    fn test_notify_respects_snooze() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Set up a cache with snooze active
        let cache = PruneCache {
            last_check: Utc::now(),
            stale_count: 5,
            snoozed_until: Some(Utc::now() + Duration::days(1)),
        };
        save_cache(&config, &cache).unwrap();

        let db_file = temp_dir.path().join("aliases.toml");
        let db = crate::database::Database::load_from_path(&db_file).unwrap();

        // Should return early due to snooze (no notification)
        notify_if_stale_aliases(&config, &db);
        // Test passes if no panic - we can't easily test stderr output
    }
}
