//! Self-update functionality for goto

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::PathBuf;

use crate::config::Config;

const GITHUB_API_URL: &str = "https://api.github.com/repos/anttilinno/goto/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Cached update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCache {
    pub last_check: DateTime<Utc>,
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
}

impl Default for UpdateCache {
    fn default() -> Self {
        Self {
            last_check: DateTime::from_timestamp(0, 0).unwrap(),
            latest_version: None,
            download_url: None,
            checksum: None,
        }
    }
}

/// GitHub release asset information
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// GitHub release information
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

/// Get the path to the update cache file
fn cache_path(config: &Config) -> PathBuf {
    config.database_path.join("update_cache.json")
}

/// Load the update cache from disk
fn load_cache(config: &Config) -> UpdateCache {
    let path = cache_path(config);
    if !path.exists() {
        return UpdateCache::default();
    }

    match File::open(&path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_default()
        }
        Err(_) => UpdateCache::default(),
    }
}

/// Save the update cache to disk
fn save_cache(config: &Config, cache: &UpdateCache) -> Result<(), Box<dyn Error>> {
    config.ensure_dirs()?;
    let path = cache_path(config);
    let file = File::create(&path)?;
    serde_json::to_writer_pretty(file, cache)?;
    Ok(())
}

/// Parse version string, stripping 'v' prefix if present
fn parse_version(version: &str) -> &str {
    version.strip_prefix('v').unwrap_or(version)
}

/// Compare two version strings (semver-like)
/// Returns true if version_a > version_b
fn is_newer_version(version_a: &str, version_b: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        parse_version(v)
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let a = parse(version_a);
    let b = parse(version_b);

    for (va, vb) in a.iter().zip(b.iter()) {
        if va > vb {
            return true;
        }
        if va < vb {
            return false;
        }
    }

    a.len() > b.len()
}

/// Check for updates from GitHub
fn fetch_latest_release() -> Result<GitHubRelease, Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("goto/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(GITHUB_API_URL).send()?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status {}", response.status()).into());
    }

    let release: GitHubRelease = response.json()?;
    Ok(release)
}

/// Fetch the checksum file and extract the checksum for the binary
fn fetch_checksum(assets: &[GitHubAsset]) -> Option<String> {
    let checksum_asset = assets.iter().find(|a| a.name == "checksums.txt")?;

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("goto/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .ok()?;

    let content = client
        .get(&checksum_asset.browser_download_url)
        .send()
        .ok()?
        .text()
        .ok()?;

    // Parse checksums.txt format: "hash  filename"
    for line in content.lines() {
        if line.contains("goto-linux-amd64") {
            return line.split_whitespace().next().map(String::from);
        }
    }
    None
}

/// Get the appropriate binary asset name for the current platform
fn get_binary_asset_name() -> Option<&'static str> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        Some("goto-linux-amd64")
    }
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    {
        None
    }
}

/// Check for updates and update the cache
pub fn check_for_updates(
    config: &Config,
    force: bool,
) -> Result<Option<String>, Box<dyn Error>> {
    let mut cache = load_cache(config);
    let check_interval = Duration::hours(config.user.update.check_interval_hours as i64);

    // Skip if checked recently (unless forced)
    if !force && Utc::now() - cache.last_check < check_interval {
        return Ok(cache.latest_version.filter(|v| is_newer_version(v, CURRENT_VERSION)));
    }

    // Fetch latest release
    let release = fetch_latest_release()?;
    let latest_version = parse_version(&release.tag_name).to_string();

    // Find the appropriate binary asset
    let asset_name = get_binary_asset_name();
    let download_url = asset_name
        .and_then(|name| release.assets.iter().find(|a| a.name == name))
        .map(|a| a.browser_download_url.clone());

    // Fetch checksum
    let checksum = fetch_checksum(&release.assets);

    // Update cache
    cache.last_check = Utc::now();
    cache.latest_version = Some(latest_version.clone());
    cache.download_url = download_url;
    cache.checksum = checksum;
    save_cache(config, &cache)?;

    if is_newer_version(&latest_version, CURRENT_VERSION) {
        Ok(Some(latest_version))
    } else {
        Ok(None)
    }
}

/// Show a notification if an update is available (non-blocking, best-effort)
pub fn notify_if_update_available(config: &Config) {
    if !config.user.update.auto_check {
        return;
    }

    let cache = load_cache(config);

    // Check if we should perform a background check
    let check_interval = Duration::hours(config.user.update.check_interval_hours as i64);
    if Utc::now() - cache.last_check >= check_interval {
        // Try to check for updates, but don't block on errors
        let _ = check_for_updates(config, false);
        return;
    }

    // Show notification if update is available
    if let Some(ref latest) = cache.latest_version {
        if is_newer_version(latest, CURRENT_VERSION) {
            eprintln!(
                "Update available: {} (current: {}). Run 'goto --update' to upgrade.",
                latest, CURRENT_VERSION
            );
        }
    }
}

/// Get the path to the currently running binary
fn get_current_binary_path() -> Result<PathBuf, Box<dyn Error>> {
    std::env::current_exe().map_err(|e| e.into())
}

/// Calculate SHA256 checksum of a file
fn calculate_sha256(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Simple SHA256 implementation using standard library would be ideal,
    // but we'll use a basic approach for verification
    // For production, consider adding sha2 crate
    // For now, we'll use system sha256sum command
    use std::process::Command;

    let output = Command::new("sha256sum").arg(path).output()?;

    if !output.status.success() {
        return Err("Failed to calculate checksum".into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    output_str
        .split_whitespace()
        .next()
        .map(String::from)
        .ok_or_else(|| "Invalid checksum output".into())
}

/// Perform the self-update
pub fn perform_update(config: &Config) -> Result<(), Box<dyn Error>> {
    println!("Checking for updates...");

    // Force a fresh check
    let latest = check_for_updates(config, true)?;

    match latest {
        None => {
            println!("You are running the latest version ({}).", CURRENT_VERSION);
            return Ok(());
        }
        Some(version) => {
            println!("New version available: {} (current: {})", version, CURRENT_VERSION);
        }
    }

    let cache = load_cache(config);

    // Verify we have a download URL
    let download_url = cache
        .download_url
        .ok_or("No download URL available for your platform")?;

    // Get current binary path
    let current_binary = get_current_binary_path()?;

    // Check if we have write permissions
    let parent_dir = current_binary.parent().ok_or("Cannot determine binary directory")?;
    if fs::metadata(parent_dir)?.permissions().readonly() {
        return Err("Cannot update: binary directory is read-only. Try running with elevated permissions.".into());
    }

    println!("Downloading {}...", cache.latest_version.as_deref().unwrap_or("update"));

    // Download to temp file
    let temp_path = parent_dir.join(".goto-bin.new");

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("goto/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let response = client.get(&download_url).send()?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status {}", response.status()).into());
    }

    let bytes = response.bytes()?;
    let mut file = File::create(&temp_path)?;
    file.write_all(&bytes)?;
    drop(file);

    // Verify checksum if available
    if let Some(expected_checksum) = &cache.checksum {
        print!("Verifying checksum...");
        io::stdout().flush()?;

        let actual_checksum = calculate_sha256(&temp_path)?;

        if actual_checksum != *expected_checksum {
            fs::remove_file(&temp_path)?;
            return Err(format!(
                "Checksum verification failed!\nExpected: {}\nGot: {}",
                expected_checksum, actual_checksum
            )
            .into());
        }
        println!(" OK");
    } else {
        eprintln!("Warning: No checksum available, skipping verification");
    }

    // Make the new binary executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms)?;
    }

    // Rename current binary to .old
    let backup_path = parent_dir.join(".goto-bin.old");
    if backup_path.exists() {
        fs::remove_file(&backup_path)?;
    }

    println!("Installing update...");

    // Rename current -> backup
    fs::rename(&current_binary, &backup_path)?;

    // Rename new -> current
    match fs::rename(&temp_path, &current_binary) {
        Ok(()) => {
            // Clean up backup
            let _ = fs::remove_file(&backup_path);
        }
        Err(e) => {
            // Try to restore from backup
            eprintln!("Error during update: {}", e);
            if let Err(restore_err) = fs::rename(&backup_path, &current_binary) {
                eprintln!(
                    "CRITICAL: Failed to restore backup: {}. Manual intervention required!",
                    restore_err
                );
            }
            return Err(e.into());
        }
    }

    println!(
        "Update complete! goto {} -> {}",
        CURRENT_VERSION,
        cache.latest_version.as_deref().unwrap_or("unknown")
    );
    println!("Restart your shell to use the new version.");

    Ok(())
}

/// Show version with update status
pub fn version_with_update_status(config: &Config) -> String {
    let cache = load_cache(config);

    if let Some(ref latest) = cache.latest_version {
        if is_newer_version(latest, CURRENT_VERSION) {
            return format!(
                "goto version {} (update available: {})",
                CURRENT_VERSION, latest
            );
        }
    }

    format!("goto version {}", CURRENT_VERSION)
}

/// Get the current version
pub fn current_version() -> &'static str {
    CURRENT_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UserConfig;

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
    fn test_parse_version() {
        assert_eq!(parse_version("v1.2.3"), "1.2.3");
        assert_eq!(parse_version("1.2.3"), "1.2.3");
        assert_eq!(parse_version("v0.1.0"), "0.1.0");
    }

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.5.0", "1.4.0"));
        assert!(is_newer_version("2.0.0", "1.9.9"));
        assert!(is_newer_version("1.4.1", "1.4.0"));
        assert!(is_newer_version("1.4.0.1", "1.4.0"));

        assert!(!is_newer_version("1.4.0", "1.5.0"));
        assert!(!is_newer_version("1.4.0", "1.4.0"));
        assert!(!is_newer_version("1.3.9", "1.4.0"));
    }

    #[test]
    fn test_is_newer_version_with_prefix() {
        assert!(is_newer_version("v1.5.0", "1.4.0"));
        assert!(is_newer_version("1.5.0", "v1.4.0"));
        assert!(is_newer_version("v1.5.0", "v1.4.0"));
    }

    #[test]
    fn test_update_cache_default() {
        let cache = UpdateCache::default();
        assert!(cache.latest_version.is_none());
        assert!(cache.download_url.is_none());
        assert!(cache.checksum.is_none());
    }

    #[test]
    fn test_update_cache_serialization() {
        let cache = UpdateCache {
            last_check: Utc::now(),
            latest_version: Some("1.5.0".to_string()),
            download_url: Some("https://example.com/binary".to_string()),
            checksum: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&cache).unwrap();
        let deserialized: UpdateCache = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.latest_version, cache.latest_version);
        assert_eq!(deserialized.download_url, cache.download_url);
        assert_eq!(deserialized.checksum, cache.checksum);
    }

    #[test]
    fn test_get_binary_asset_name() {
        let name = get_binary_asset_name();
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        assert_eq!(name, Some("goto-linux-amd64"));
    }

    #[test]
    fn test_current_version() {
        let version = current_version();
        assert!(!version.is_empty());
        // Should be a valid semver-like format
        assert!(version.contains('.'));
    }

    #[test]
    fn test_cache_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = test_config(temp_dir.path());
        let path = cache_path(&config);
        assert_eq!(path, temp_dir.path().join("update_cache.json"));
    }

    #[test]
    fn test_load_cache_nonexistent() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = test_config(temp_dir.path());

        let cache = load_cache(&config);
        assert!(cache.latest_version.is_none());
    }

    #[test]
    fn test_save_and_load_cache() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = test_config(temp_dir.path());

        let cache = UpdateCache {
            last_check: Utc::now(),
            latest_version: Some("2.0.0".to_string()),
            download_url: Some("https://example.com/download".to_string()),
            checksum: Some("sha256hash".to_string()),
        };

        save_cache(&config, &cache).unwrap();

        let loaded = load_cache(&config);
        assert_eq!(loaded.latest_version, Some("2.0.0".to_string()));
        assert_eq!(loaded.download_url, Some("https://example.com/download".to_string()));
        assert_eq!(loaded.checksum, Some("sha256hash".to_string()));
    }

    #[test]
    fn test_load_cache_invalid_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = test_config(temp_dir.path());

        // Write invalid JSON
        let cache_file = temp_dir.path().join("update_cache.json");
        fs::write(&cache_file, "not valid json").unwrap();

        // Should return default cache on parse error
        let cache = load_cache(&config);
        assert!(cache.latest_version.is_none());
    }

    #[test]
    fn test_version_with_update_status_no_cache() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = test_config(temp_dir.path());

        let version = version_with_update_status(&config);
        assert!(version.starts_with("goto version "));
        assert!(!version.contains("update available"));
    }

    #[test]
    fn test_version_with_update_status_with_update() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = test_config(temp_dir.path());

        // Save a cache indicating a newer version
        let cache = UpdateCache {
            last_check: Utc::now(),
            latest_version: Some("99.0.0".to_string()), // Very high version
            download_url: None,
            checksum: None,
        };
        save_cache(&config, &cache).unwrap();

        let version = version_with_update_status(&config);
        assert!(version.contains("update available: 99.0.0"));
    }

    #[test]
    fn test_version_with_update_status_same_version() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = test_config(temp_dir.path());

        // Save a cache with the current version
        let cache = UpdateCache {
            last_check: Utc::now(),
            latest_version: Some(CURRENT_VERSION.to_string()),
            download_url: None,
            checksum: None,
        };
        save_cache(&config, &cache).unwrap();

        let version = version_with_update_status(&config);
        assert!(!version.contains("update available"));
    }

    #[test]
    fn test_version_with_update_status_older_version() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config = test_config(temp_dir.path());

        // Save a cache with an older version (shouldn't happen, but test anyway)
        let cache = UpdateCache {
            last_check: Utc::now(),
            latest_version: Some("0.0.1".to_string()),
            download_url: None,
            checksum: None,
        };
        save_cache(&config, &cache).unwrap();

        let version = version_with_update_status(&config);
        assert!(!version.contains("update available"));
    }

    #[test]
    fn test_notify_disabled_when_auto_check_false() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut config = test_config(temp_dir.path());
        config.user.update.auto_check = false;

        // Save a cache indicating update available
        let cache = UpdateCache {
            last_check: Utc::now(),
            latest_version: Some("99.0.0".to_string()),
            download_url: None,
            checksum: None,
        };
        save_cache(&config, &cache).unwrap();

        // Should not panic and return early (no notification when disabled)
        notify_if_update_available(&config);
    }
}
