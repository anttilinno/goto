//! Registration commands: register, unregister, rename

use std::collections::HashSet;

use crate::alias::{validate_alias, validate_tag, Alias, AliasError};
use crate::config::expand_path;
use crate::database::Database;

/// Register a new alias for a directory
pub fn register(db: &mut Database, name: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    register_with_tags(db, name, path, &[], false)
}

/// Register a new alias with optional tags
///
/// # Arguments
/// * `db` - The alias database
/// * `name` - The alias name
/// * `path` - The directory path
/// * `tags` - Tags to add to the alias
/// * `_force` - If true, skip confirmation for new tags (used in future plan)
pub fn register_with_tags(
    db: &mut Database,
    name: &str,
    path: &str,
    tags: &[String],
    _force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate alias name
    validate_alias(name)?;

    // Validate and normalize tags
    let normalized_tags = validate_and_normalize_tags(tags)?;

    // Expand and validate directory
    let expanded_path = expand_path(path)?;
    let path_str = expanded_path.to_string_lossy().to_string();

    // Check directory exists
    if !expanded_path.exists() {
        return Err(AliasError::DirectoryNotFound(path_str).into());
    }
    if !expanded_path.is_dir() {
        return Err(format!("not a directory: {}", path_str).into());
    }

    // Add alias with tags
    let alias = Alias {
        name: name.to_string(),
        path: path_str.clone(),
        tags: Vec::new(),
        use_count: 0,
        last_used: None,
        created_at: chrono::Utc::now(),
    };

    db.add_with_tags(alias, normalized_tags.clone())?;
    db.save()?;

    if !normalized_tags.is_empty() {
        println!(
            "Registered '{}' -> {} [{}]",
            name,
            path_str,
            normalized_tags.join(", ")
        );
    } else {
        println!("Registered '{}' -> {}", name, path_str);
    }

    Ok(())
}

/// Validate tags and convert to lowercase, removing duplicates
fn validate_and_normalize_tags(tags: &[String]) -> Result<Vec<String>, AliasError> {
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();

    for tag in tags {
        let tag = tag.trim().to_lowercase();
        if tag.is_empty() {
            continue;
        }
        validate_tag(&tag)?;
        if !seen.contains(&tag) {
            seen.insert(tag.clone());
            normalized.push(tag);
        }
    }

    Ok(normalized)
}

/// Unregister (remove) an alias
pub fn unregister(db: &mut Database, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if db.remove(name).is_some() {
        db.save()?;
        println!("Unregistered '{}'", name);
        Ok(())
    } else {
        Err(AliasError::NotFound(name.to_string()).into())
    }
}

/// Rename an alias while preserving all metadata
pub fn rename(
    db: &mut Database,
    old_name: &str,
    new_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate new alias name
    validate_alias(new_name)?;

    db.rename_alias(old_name, new_name)?;
    db.save()?;

    println!("Renamed alias '{}' to '{}'", old_name, new_name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_db() -> (Database, NamedTempFile) {
        let file = NamedTempFile::new().unwrap();
        let db = Database::load_from_path(file.path()).unwrap();
        (db, file)
    }

    #[test]
    fn test_register() {
        let (mut db, _file) = create_test_db();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_string_lossy().to_string();

        let result = register(&mut db, "test", &path);
        assert!(result.is_ok());
        assert!(db.contains("test"));
    }

    #[test]
    fn test_register_duplicate() {
        let (mut db, _file) = create_test_db();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_string_lossy().to_string();

        register(&mut db, "test", &path).unwrap();
        let result = register(&mut db, "test", &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_nonexistent_path() {
        let (mut db, _file) = create_test_db();
        let result = register(&mut db, "test", "/nonexistent/path/12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_register_validates_alias() {
        let (mut db, _file) = create_test_db();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_string_lossy().to_string();

        // Invalid alias starting with dash
        let result = register(&mut db, "-invalid", &path);
        assert!(result.is_err());

        // Invalid alias with spaces
        let result = register(&mut db, "hello world", &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_with_tags() {
        let (mut db, _file) = create_test_db();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_string_lossy().to_string();

        let tags = vec!["Work".to_string(), "important".to_string()];
        let result = register_with_tags(&mut db, "test", &path, &tags, false);
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        // Tags should be normalized to lowercase
        assert!(alias.tags.contains(&"work".to_string()));
        assert!(alias.tags.contains(&"important".to_string()));
    }

    #[test]
    fn test_register_with_tags_validates_tags() {
        let (mut db, _file) = create_test_db();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_string_lossy().to_string();

        // Invalid tag starting with dash
        let tags = vec!["-invalid".to_string()];
        let result = register_with_tags(&mut db, "test", &path, &tags, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_with_tags_deduplicates() {
        let (mut db, _file) = create_test_db();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_string_lossy().to_string();

        // Same tag with different cases should be deduplicated
        let tags = vec!["Work".to_string(), "WORK".to_string(), "work".to_string()];
        let result = register_with_tags(&mut db, "test", &path, &tags, false);
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert_eq!(alias.tags.len(), 1);
        assert!(alias.tags.contains(&"work".to_string()));
    }

    #[test]
    fn test_register_with_empty_tags_skipped() {
        let (mut db, _file) = create_test_db();
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_string_lossy().to_string();

        let tags = vec!["work".to_string(), "".to_string(), "  ".to_string()];
        let result = register_with_tags(&mut db, "test", &path, &tags, false);
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert_eq!(alias.tags.len(), 1);
    }

    #[test]
    fn test_register_expands_tilde() {
        let (mut db, _file) = create_test_db();
        // This test checks that ~ is expanded - we can't easily test the result
        // but we can verify it doesn't crash
        let result = register(&mut db, "home", "~");
        // May succeed or fail depending on whether home dir exists
        // The important thing is it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_register_not_a_directory() {
        let (mut db, _file) = create_test_db();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_string_lossy().to_string();

        let result = register(&mut db, "test", &path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a directory"));
    }

    #[test]
    fn test_unregister() {
        let (mut db, _file) = create_test_db();
        db.insert(Alias::new("test", "/tmp").unwrap());

        let result = unregister(&mut db, "test");
        assert!(result.is_ok());
        assert!(!db.contains("test"));
    }

    #[test]
    fn test_unregister_not_found() {
        let (mut db, _file) = create_test_db();
        let result = unregister(&mut db, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename() {
        let (mut db, _file) = create_test_db();
        let mut alias = Alias::new("old", "/tmp").unwrap();
        alias.add_tag("important");
        alias.record_use();
        db.insert(alias);

        let result = rename(&mut db, "old", "new");
        assert!(result.is_ok());
        assert!(!db.contains("old"));
        assert!(db.contains("new"));

        let renamed = db.get("new").unwrap();
        assert!(renamed.has_tag("important"));
        assert_eq!(renamed.use_count, 1);
    }

    #[test]
    fn test_rename_not_found() {
        let (mut db, _file) = create_test_db();
        let result = rename(&mut db, "nonexistent", "new");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_target_exists() {
        let (mut db, _file) = create_test_db();
        db.insert(Alias::new("old", "/tmp").unwrap());
        db.insert(Alias::new("new", "/tmp").unwrap());

        let result = rename(&mut db, "old", "new");
        assert!(result.is_err());
    }

    #[test]
    fn test_rename_validates_new_name() {
        let (mut db, _file) = create_test_db();
        db.insert(Alias::new("old", "/tmp").unwrap());

        // Invalid name starting with dash
        let result = rename(&mut db, "old", "-invalid");
        assert!(result.is_err());

        // Invalid name with spaces
        let result = rename(&mut db, "old", "hello world");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_and_normalize_tags() {
        // Valid tags
        let tags = vec!["Work".to_string(), "IMPORTANT".to_string()];
        let result = validate_and_normalize_tags(&tags).unwrap();
        assert_eq!(result, vec!["work", "important"]);

        // Deduplicate
        let tags = vec!["work".to_string(), "Work".to_string()];
        let result = validate_and_normalize_tags(&tags).unwrap();
        assert_eq!(result, vec!["work"]);

        // Skip empty
        let tags = vec!["work".to_string(), "".to_string()];
        let result = validate_and_normalize_tags(&tags).unwrap();
        assert_eq!(result, vec!["work"]);

        // Invalid tag
        let tags = vec!["-invalid".to_string()];
        let result = validate_and_normalize_tags(&tags);
        assert!(result.is_err());
    }
}
