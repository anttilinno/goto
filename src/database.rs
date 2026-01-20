//! TOML-based alias storage with metadata

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::alias::{Alias, AliasError};
use crate::config::{Config, ConfigError};
use crate::fuzzy;

/// Errors that can occur during database operations
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("config error: {0}")]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Alias(#[from] AliasError),
}

/// Database file format - array-based structure
#[derive(Debug, Serialize, Deserialize, Default)]
struct DatabaseFile {
    #[serde(default)]
    aliases: Vec<Alias>,
}

/// In-memory database with file persistence
#[derive(Debug)]
pub struct Database {
    /// Path to the TOML database file
    toml_path: PathBuf,
    /// Path to old text file (for migration)
    text_path: PathBuf,
    /// Aliases stored by name for fast lookup
    aliases: HashMap<String, Alias>,
    /// Whether the database has unsaved changes
    dirty: bool,
}

impl Database {
    /// Load the database from the configured path
    pub fn load(config: &Config) -> Result<Self, DatabaseError> {
        config.ensure_dirs()?;
        Self::load_from_path(&config.aliases_path)
    }

    /// Load the database from a specific path
    /// The path should be the base path (e.g., ~/.config/goto/aliases)
    /// The TOML file will be at path + ".toml"
    pub fn load_from_path(path: &Path) -> Result<Self, DatabaseError> {
        let toml_path = path.with_extension("toml");
        let text_path = path.to_path_buf();

        let mut db = Self {
            toml_path,
            text_path,
            aliases: HashMap::new(),
            dirty: false,
        };

        db.load_entries()?;
        Ok(db)
    }

    /// Load entries from storage (TOML or migrate from text)
    fn load_entries(&mut self) -> Result<(), DatabaseError> {
        // Check if TOML file exists
        if self.toml_path.exists() {
            self.load_toml()?;
            return Ok(());
        }

        // Check if old text file exists and migrate
        if self.text_path.exists() {
            self.migrate_from_text_format()?;
            return Ok(());
        }

        // No database exists, start empty
        Ok(())
    }

    /// Load aliases from TOML file
    fn load_toml(&mut self) -> Result<(), DatabaseError> {
        let content = fs::read_to_string(&self.toml_path)?;
        let db_file: DatabaseFile = toml::from_str(&content)?;

        self.aliases.clear();
        for alias in db_file.aliases {
            self.aliases.insert(alias.name.clone(), alias);
        }

        Ok(())
    }

    /// Migrate from old text format to TOML
    fn migrate_from_text_format(&mut self) -> Result<(), DatabaseError> {
        let content = fs::read_to_string(&self.text_path)?;
        let now = Utc::now();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Split on first space only (path may contain spaces)
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                let alias = Alias {
                    name: parts[0].to_string(),
                    path: parts[1].to_string(),
                    tags: Vec::new(),
                    use_count: 0,
                    last_used: None,
                    created_at: now,
                };
                self.aliases.insert(alias.name.clone(), alias);
            }
        }

        // Save as TOML
        self.dirty = true;
        self.save()?;

        // Backup old file
        let backup_path = self.text_path.with_extension("txt.bak");
        let _ = fs::rename(&self.text_path, backup_path);

        Ok(())
    }

    /// Save the database to disk
    pub fn save(&mut self) -> Result<(), DatabaseError> {
        if !self.dirty {
            return Ok(());
        }

        // Collect aliases into a vector sorted by name for consistent output
        let mut aliases: Vec<Alias> = self.aliases.values().cloned().collect();
        aliases.sort_by(|a, b| a.name.cmp(&b.name));

        let db_file = DatabaseFile { aliases };
        let content = toml::to_string_pretty(&db_file)?;

        // Ensure parent directory exists
        if let Some(parent) = self.toml_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.toml_path, content)?;
        self.dirty = false;
        Ok(())
    }

    /// Get an alias by name
    pub fn get(&self, name: &str) -> Option<&Alias> {
        self.aliases.get(name)
    }

    /// Get a mutable reference to an alias by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Alias> {
        self.dirty = true;
        self.aliases.get_mut(name)
    }

    /// Insert or update an alias
    pub fn insert(&mut self, alias: Alias) {
        self.dirty = true;
        self.aliases.insert(alias.name.clone(), alias);
    }

    /// Add a new alias (fails if exists)
    pub fn add(&mut self, alias: Alias) -> Result<(), DatabaseError> {
        if self.aliases.contains_key(&alias.name) {
            return Err(AliasError::AlreadyExists(alias.name).into());
        }
        self.insert(alias);
        Ok(())
    }

    /// Add a new alias with tags (fails if exists)
    pub fn add_with_tags(&mut self, mut alias: Alias, mut tags: Vec<String>) -> Result<(), DatabaseError> {
        if self.aliases.contains_key(&alias.name) {
            return Err(AliasError::AlreadyExists(alias.name).into());
        }
        tags.sort();
        alias.tags = tags;
        self.insert(alias);
        Ok(())
    }

    /// Remove an alias by name
    pub fn remove(&mut self, name: &str) -> Option<Alias> {
        self.dirty = true;
        self.aliases.remove(name)
    }

    /// Check if an alias exists
    pub fn contains(&self, name: &str) -> bool {
        self.aliases.contains_key(name)
    }

    /// Get all aliases
    pub fn all(&self) -> impl Iterator<Item = &Alias> {
        self.aliases.values()
    }

    /// Get all alias names
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.aliases.keys().map(|s| s.as_str())
    }

    /// Get all alias names as a vector
    pub fn list_names(&self) -> Vec<String> {
        self.aliases.keys().cloned().collect()
    }

    /// Get the number of aliases
    pub fn len(&self) -> usize {
        self.aliases.len()
    }

    /// Check if the database is empty
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
    }

    /// Record usage of an alias (increment use_count, update last_used)
    pub fn record_usage(&mut self, name: &str) -> Result<(), DatabaseError> {
        if let Some(alias) = self.aliases.get_mut(name) {
            alias.record_use();
            self.dirty = true;
            Ok(())
        } else {
            Err(AliasError::NotFound(name.to_string()).into())
        }
    }

    /// Rename an alias while preserving all metadata
    pub fn rename_alias(&mut self, old_name: &str, new_name: &str) -> Result<(), DatabaseError> {
        // Check new name doesn't exist
        if self.aliases.contains_key(new_name) {
            return Err(AliasError::AlreadyExists(new_name.to_string()).into());
        }

        // Remove old entry
        let mut alias = self
            .aliases
            .remove(old_name)
            .ok_or_else(|| AliasError::NotFound(old_name.to_string()))?;

        // Update name and insert with new key
        alias.name = new_name.to_string();
        self.aliases.insert(new_name.to_string(), alias);
        self.dirty = true;
        Ok(())
    }

    /// Add a tag to an alias
    pub fn add_tag(&mut self, alias_name: &str, tag: &str) -> Result<(), DatabaseError> {
        if let Some(alias) = self.aliases.get_mut(alias_name) {
            alias.add_tag(tag);
            self.dirty = true;
            Ok(())
        } else {
            Err(AliasError::NotFound(alias_name.to_string()).into())
        }
    }

    /// Remove a tag from an alias
    pub fn remove_tag(&mut self, alias_name: &str, tag: &str) -> Result<(), DatabaseError> {
        if let Some(alias) = self.aliases.get_mut(alias_name) {
            alias.remove_tag(tag);
            self.dirty = true;
            Ok(())
        } else {
            Err(AliasError::NotFound(alias_name.to_string()).into())
        }
    }

    /// Set all tags on an alias (replacing existing)
    pub fn set_tags(&mut self, alias_name: &str, tags: Vec<String>) -> Result<(), DatabaseError> {
        if let Some(alias) = self.aliases.get_mut(alias_name) {
            alias.tags = tags;
            alias.tags.sort();
            self.dirty = true;
            Ok(())
        } else {
            Err(AliasError::NotFound(alias_name.to_string()).into())
        }
    }

    /// Get all unique tags with their counts
    pub fn get_all_tags(&self) -> HashMap<String, usize> {
        let mut tag_counts = HashMap::new();
        for alias in self.aliases.values() {
            for tag in &alias.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        tag_counts
    }

    /// Get all unique tags across all aliases (sorted)
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self
            .aliases
            .values()
            .flat_map(|a| a.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Clear recent history (reset last_used for all aliases)
    pub fn clear_recent_history(&mut self) -> Result<(), DatabaseError> {
        for alias in self.aliases.values_mut() {
            alias.last_used = None;
        }
        self.dirty = true;
        Ok(())
    }

    /// Find similar alias names using fuzzy matching
    pub fn find_similar(&self, query: &str, threshold: f64) -> Vec<String> {
        let names = self.list_names();
        fuzzy::find_similar_names(query, &names, threshold)
    }

    /// Export the database as TOML string
    pub fn export_toml(&self) -> Result<String, DatabaseError> {
        let mut aliases: Vec<Alias> = self.aliases.values().cloned().collect();
        aliases.sort_by(|a, b| a.name.cmp(&b.name));
        let db_file = DatabaseFile { aliases };
        Ok(toml::to_string_pretty(&db_file)?)
    }

    /// Import aliases from TOML string
    pub fn import_toml(&mut self, content: &str) -> Result<usize, DatabaseError> {
        let db_file: DatabaseFile = toml::from_str(content)?;
        let count = db_file.aliases.len();
        for alias in db_file.aliases {
            self.aliases.insert(alias.name.clone(), alias);
        }
        self.dirty = true;
        Ok(count)
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // Try to save on drop, but ignore errors
        let _ = self.save();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_db() -> (Database, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("aliases");
        let db = Database::load_from_path(&path).unwrap();
        (db, dir)
    }

    #[test]
    fn test_empty_database() {
        let (db, _dir) = create_test_db();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let (mut db, _dir) = create_test_db();
        let alias = Alias::new("test", "/tmp/test").unwrap();
        db.insert(alias);

        assert!(!db.is_empty());
        assert_eq!(db.len(), 1);
        assert!(db.contains("test"));

        let retrieved = db.get("test").unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.path, "/tmp/test");
    }

    #[test]
    fn test_add_fails_if_exists() {
        let (mut db, _dir) = create_test_db();
        let alias1 = Alias::new("test", "/tmp/test1").unwrap();
        let alias2 = Alias::new("test", "/tmp/test2").unwrap();

        db.add(alias1).unwrap();
        let result = db.add(alias2);

        assert!(matches!(result, Err(DatabaseError::Alias(AliasError::AlreadyExists(_)))));
    }

    #[test]
    fn test_add_with_tags() {
        let (mut db, _dir) = create_test_db();
        let alias = Alias::new("test", "/tmp/test").unwrap();
        let tags = vec!["work".to_string(), "important".to_string()];

        db.add_with_tags(alias, tags).unwrap();

        let retrieved = db.get("test").unwrap();
        assert_eq!(retrieved.tags, vec!["important", "work"]); // sorted
    }

    #[test]
    fn test_remove() {
        let (mut db, _dir) = create_test_db();
        let alias = Alias::new("test", "/tmp/test").unwrap();
        db.insert(alias);

        let removed = db.remove("test").unwrap();
        assert_eq!(removed.name, "test");
        assert!(db.is_empty());
    }

    #[test]
    fn test_record_usage() {
        let (mut db, _dir) = create_test_db();
        let alias = Alias::new("test", "/tmp/test").unwrap();
        db.insert(alias);

        assert_eq!(db.get("test").unwrap().use_count, 0);
        assert!(db.get("test").unwrap().last_used.is_none());

        db.record_usage("test").unwrap();

        assert_eq!(db.get("test").unwrap().use_count, 1);
        assert!(db.get("test").unwrap().last_used.is_some());
    }

    #[test]
    fn test_record_usage_not_found() {
        let (mut db, _dir) = create_test_db();
        let result = db.record_usage("nonexistent");
        assert!(matches!(result, Err(DatabaseError::Alias(AliasError::NotFound(_)))));
    }

    #[test]
    fn test_rename_alias() {
        let (mut db, _dir) = create_test_db();
        let mut alias = Alias::new("old", "/tmp/test").unwrap();
        alias.use_count = 5;
        alias.add_tag("work");
        db.insert(alias);

        db.rename_alias("old", "new").unwrap();

        assert!(!db.contains("old"));
        assert!(db.contains("new"));

        let renamed = db.get("new").unwrap();
        assert_eq!(renamed.use_count, 5);
        assert!(renamed.has_tag("work"));
    }

    #[test]
    fn test_rename_alias_to_existing() {
        let (mut db, _dir) = create_test_db();
        let alias1 = Alias::new("first", "/tmp/first").unwrap();
        let alias2 = Alias::new("second", "/tmp/second").unwrap();
        db.insert(alias1);
        db.insert(alias2);

        let result = db.rename_alias("first", "second");
        assert!(matches!(result, Err(DatabaseError::Alias(AliasError::AlreadyExists(_)))));
    }

    #[test]
    fn test_tag_operations() {
        let (mut db, _dir) = create_test_db();
        let alias = Alias::new("test", "/tmp/test").unwrap();
        db.insert(alias);

        db.add_tag("test", "work").unwrap();
        assert!(db.get("test").unwrap().has_tag("work"));

        db.add_tag("test", "important").unwrap();
        assert!(db.get("test").unwrap().has_tag("important"));

        db.remove_tag("test", "work").unwrap();
        assert!(!db.get("test").unwrap().has_tag("work"));
        assert!(db.get("test").unwrap().has_tag("important"));
    }

    #[test]
    fn test_set_tags() {
        let (mut db, _dir) = create_test_db();
        let mut alias = Alias::new("test", "/tmp/test").unwrap();
        alias.add_tag("old");
        db.insert(alias);

        db.set_tags("test", vec!["new".to_string(), "tags".to_string()]).unwrap();

        let retrieved = db.get("test").unwrap();
        assert!(!retrieved.has_tag("old"));
        assert!(retrieved.has_tag("new"));
        assert!(retrieved.has_tag("tags"));
    }

    #[test]
    fn test_get_all_tags() {
        let (mut db, _dir) = create_test_db();

        let mut alias1 = Alias::new("test1", "/tmp/test1").unwrap();
        alias1.add_tag("work");
        alias1.add_tag("important");
        db.insert(alias1);

        let mut alias2 = Alias::new("test2", "/tmp/test2").unwrap();
        alias2.add_tag("work");
        alias2.add_tag("personal");
        db.insert(alias2);

        let tag_counts = db.get_all_tags();
        assert_eq!(tag_counts.get("work"), Some(&2));
        assert_eq!(tag_counts.get("important"), Some(&1));
        assert_eq!(tag_counts.get("personal"), Some(&1));
    }

    #[test]
    fn test_all_tags() {
        let (mut db, _dir) = create_test_db();

        let mut alias1 = Alias::new("test1", "/tmp/test1").unwrap();
        alias1.add_tag("work");
        alias1.add_tag("important");
        db.insert(alias1);

        let mut alias2 = Alias::new("test2", "/tmp/test2").unwrap();
        alias2.add_tag("work");
        alias2.add_tag("personal");
        db.insert(alias2);

        let tags = db.all_tags();
        assert_eq!(tags, vec!["important", "personal", "work"]);
    }

    #[test]
    fn test_clear_recent_history() {
        let (mut db, _dir) = create_test_db();
        let alias = Alias::new("test", "/tmp/test").unwrap();
        db.insert(alias);
        db.record_usage("test").unwrap();

        assert!(db.get("test").unwrap().last_used.is_some());

        db.clear_recent_history().unwrap();

        assert!(db.get("test").unwrap().last_used.is_none());
    }

    #[test]
    fn test_find_similar() {
        let (mut db, _dir) = create_test_db();
        db.insert(Alias::new("projects", "/tmp/projects").unwrap());
        db.insert(Alias::new("personal", "/tmp/personal").unwrap());
        db.insert(Alias::new("work", "/tmp/work").unwrap());

        let similar = db.find_similar("proj", 0.3);
        assert!(similar.contains(&"projects".to_string()));
    }

    #[test]
    fn test_save_and_reload() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("aliases");

        {
            let mut db = Database::load_from_path(&path).unwrap();
            let mut alias = Alias::new("test", "/tmp/test").unwrap();
            alias.add_tag("work");
            alias.use_count = 5;
            db.insert(alias);
            db.save().unwrap();
        }

        let db = Database::load_from_path(&path).unwrap();
        assert_eq!(db.len(), 1);
        assert!(db.contains("test"));

        let alias = db.get("test").unwrap();
        assert_eq!(alias.use_count, 5);
        assert!(alias.has_tag("work"));
    }

    #[test]
    fn test_migrate_from_text_format() {
        let dir = tempdir().unwrap();
        let text_path = dir.path().join("aliases");
        let toml_path = dir.path().join("aliases.toml");

        // Write old text format
        let mut file = fs::File::create(&text_path).unwrap();
        writeln!(file, "projects /home/user/projects").unwrap();
        writeln!(file, "# comment line").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "work /home/user/work").unwrap();
        drop(file);

        // Load should migrate
        let db = Database::load_from_path(&text_path).unwrap();
        assert_eq!(db.len(), 2);
        assert!(db.contains("projects"));
        assert!(db.contains("work"));

        assert_eq!(db.get("projects").unwrap().path, "/home/user/projects");

        // TOML file should exist now
        assert!(toml_path.exists());

        // Old file should be renamed to .txt.bak
        assert!(dir.path().join("aliases.txt.bak").exists());
    }

    #[test]
    fn test_export_import() {
        let (mut db, _dir) = create_test_db();
        let mut alias = Alias::new("test", "/tmp/test").unwrap();
        alias.add_tag("work");
        db.insert(alias);

        let exported = db.export_toml().unwrap();

        let (mut db2, _dir2) = create_test_db();
        let count = db2.import_toml(&exported).unwrap();
        assert_eq!(count, 1);
        assert!(db2.contains("test"));
        assert!(db2.get("test").unwrap().has_tag("work"));
    }

    #[test]
    fn test_load_existing_toml() {
        let dir = tempdir().unwrap();
        let toml_path = dir.path().join("aliases.toml");

        // Write TOML format directly
        let content = r#"[[aliases]]
name = "test"
path = "/tmp/test"
tags = ["work"]
use_count = 5
created_at = "2024-01-01T00:00:00Z"
"#;
        fs::write(&toml_path, content).unwrap();

        let base_path = dir.path().join("aliases");
        let db = Database::load_from_path(&base_path).unwrap();
        assert_eq!(db.len(), 1);

        let alias = db.get("test").unwrap();
        assert_eq!(alias.use_count, 5);
        assert!(alias.has_tag("work"));
    }

    #[test]
    fn test_load_with_config() {
        use crate::config::{Config, UserConfig};

        let dir = tempdir().unwrap();
        let config = Config {
            database_path: dir.path().to_path_buf(),
            stack_path: dir.path().join("goto_stack"),
            config_path: dir.path().join("config.toml"),
            aliases_path: dir.path().join("aliases"),
            user: UserConfig::default(),
        };

        // Test Database::load() which calls config.ensure_dirs()
        let mut db = Database::load(&config).unwrap();
        assert!(db.is_empty());

        // Add an alias and save
        let alias = Alias::new("test", "/tmp/test").unwrap();
        db.insert(alias);
        db.save().unwrap();

        // Reload using the same config
        let db2 = Database::load(&config).unwrap();
        assert!(db2.contains("test"));
    }

    #[test]
    fn test_add_tag_not_found() {
        let (mut db, _dir) = create_test_db();
        let result = db.add_tag("nonexistent", "work");
        assert!(matches!(result, Err(DatabaseError::Alias(AliasError::NotFound(_)))));
    }

    #[test]
    fn test_remove_tag_not_found() {
        let (mut db, _dir) = create_test_db();
        let result = db.remove_tag("nonexistent", "work");
        assert!(matches!(result, Err(DatabaseError::Alias(AliasError::NotFound(_)))));
    }

    #[test]
    fn test_set_tags_not_found() {
        let (mut db, _dir) = create_test_db();
        let result = db.set_tags("nonexistent", vec!["work".to_string()]);
        assert!(matches!(result, Err(DatabaseError::Alias(AliasError::NotFound(_)))));
    }

    #[test]
    fn test_auto_saves_on_drop() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("aliases");

        {
            let mut db = Database::load_from_path(&path).unwrap();
            let alias = Alias::new("dropped", "/tmp/dropped").unwrap();
            db.insert(alias);
            // Don't call save() - let Drop handle it
        }

        // Reopen and verify it was saved
        let db = Database::load_from_path(&path).unwrap();
        assert!(db.contains("dropped"));
    }

    #[test]
    fn test_dirty_flag_not_set_on_read() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("aliases");

        // Create and populate db
        {
            let mut db = Database::load_from_path(&path).unwrap();
            let alias = Alias::new("test", "/tmp/test").unwrap();
            db.insert(alias);
            db.save().unwrap();
        }

        // Get file modification time
        let toml_path = path.with_extension("toml");
        let mtime_before = fs::metadata(&toml_path).unwrap().modified().unwrap();

        // Small delay to ensure any writes would have different timestamp
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Reopen and just read - shouldn't set dirty flag
        {
            let db = Database::load_from_path(&path).unwrap();
            let _ = db.get("test");
            let _ = db.contains("test");
            let _ = db.len();
            let _ = db.is_empty();
            // On drop, should NOT write since no changes were made
        }

        // Check that file wasn't modified
        let mtime_after = fs::metadata(&toml_path).unwrap().modified().unwrap();
        assert_eq!(mtime_before, mtime_after);
    }

    #[test]
    fn test_rename_alias_not_found() {
        let (mut db, _dir) = create_test_db();
        let result = db.rename_alias("nonexistent", "newname");
        assert!(matches!(result, Err(DatabaseError::Alias(AliasError::NotFound(_)))));
    }

    #[test]
    fn test_add_with_tags_fails_if_exists() {
        let (mut db, _dir) = create_test_db();
        let alias1 = Alias::new("test", "/tmp/test1").unwrap();
        let alias2 = Alias::new("test", "/tmp/test2").unwrap();

        db.add(alias1).unwrap();
        let result = db.add_with_tags(alias2, vec!["work".to_string()]);
        assert!(matches!(result, Err(DatabaseError::Alias(AliasError::AlreadyExists(_)))));
    }
}
