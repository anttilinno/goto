//! Navigation commands: navigate, expand, completions

use std::path::Path;

use crate::alias::AliasError;
use crate::confirm;
use crate::database::Database;
use crate::fuzzy;

/// Navigate to an aliased directory
/// Prints the path for the shell function to cd to
///
/// Returns the path on success, which should be printed to stdout for the shell to cd to.
pub fn navigate(db: &mut Database, alias: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(entry) = db.get(alias) {
        // Verify directory exists
        let path = Path::new(&entry.path);
        if !path.exists() {
            return Err(AliasError::DirectoryNotFound(entry.path.clone()).into());
        }
        if !path.is_dir() {
            return Err(format!("not a directory: {}", entry.path).into());
        }

        // Get the path before mutable borrow
        let path_str = entry.path.clone();

        // Record usage
        db.record_usage(alias)?;

        // Print path for shell to cd to
        println!("{}", path_str);
        db.save()?;
        Ok(())
    } else {
        // Try fuzzy matching
        let matches: Vec<_> = fuzzy::find_matches(alias, db.names())
            .into_iter()
            .take(5)
            .collect();

        if matches.is_empty() {
            Err(format!("alias '{}' not found", alias).into())
        } else {
            // Check if best match has high confidence (>= 0.7 similarity)
            let best_match = &matches[0];
            if best_match.1 >= 700 {
                // High confidence match - prompt for confirmation
                let suggested = best_match.0.to_string();
                eprintln!("Alias '{}' not found.", alias);

                if confirm(&format!("Did you mean '{}'?", suggested), false)? {
                    // User confirmed - navigate to suggested alias
                    if let Some(entry) = db.get(&suggested) {
                        // Verify directory exists
                        let path = Path::new(&entry.path);
                        if !path.exists() {
                            return Err(AliasError::DirectoryNotFound(entry.path.clone()).into());
                        }
                        if !path.is_dir() {
                            return Err(format!("not a directory: {}", entry.path).into());
                        }

                        let path_str = entry.path.clone();
                        db.record_usage(&suggested)?;
                        println!("{}", path_str);
                        db.save()?;
                    }
                    Ok(())
                } else {
                    // User declined or non-interactive mode
                    Err("Navigation cancelled".into())
                }
            } else {
                // No match with high enough confidence - just report not found
                Err(format!("alias '{}' not found", alias).into())
            }
        }
    }
}

/// Expand an alias to its path without navigating (no side effects)
/// This is for scripts that need the raw path without recording usage.
pub fn expand(db: &Database, alias: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(entry) = db.get(alias) {
        println!("{}", entry.path);
        Ok(())
    } else {
        Err(format!("alias '{}' not found", alias).into())
    }
}

/// Generate completions for shell tab completion
pub fn completions(db: &Database, query: &str) -> Result<(), Box<dyn std::error::Error>> {
    if query.is_empty() {
        // Return all aliases
        let mut names: Vec<_> = db.names().collect();
        names.sort();
        for name in names {
            println!("{}", name);
        }
    } else {
        // Return fuzzy matches
        let matches = fuzzy::find_matches(query, db.names());
        for (name, _score) in matches {
            println!("{}", name);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::Alias;
    use tempfile::{tempdir, NamedTempFile};

    fn create_test_db() -> (Database, NamedTempFile) {
        let file = NamedTempFile::new().unwrap();
        let mut db = Database::load_from_path(file.path()).unwrap();

        db.insert(Alias::new("projects", "/home/user/projects").unwrap());
        db.insert(Alias::new("work", "/home/user/work").unwrap());
        db.insert(Alias::new("personal", "/home/user/personal").unwrap());

        (db, file)
    }

    #[test]
    fn test_expand() {
        let (db, _file) = create_test_db();
        // Just verify it doesn't panic and returns Ok
        let result = expand(&db, "projects");
        assert!(result.is_ok());
    }

    #[test]
    fn test_expand_not_found() {
        let (db, _file) = create_test_db();
        let result = expand(&db, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_completions() {
        let (db, _file) = create_test_db();
        // Just verify it doesn't panic
        let result = completions(&db, "pro");
        assert!(result.is_ok());
    }

    #[test]
    fn test_navigate_records_usage() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        // Create a real temp directory to navigate to
        let target_dir = tempdir().unwrap();
        db.insert(Alias::new("tmp", target_dir.path().to_str().unwrap()).unwrap());

        // Navigate should record usage
        let result = navigate(&mut db, "tmp");
        assert!(result.is_ok());

        let alias = db.get("tmp").unwrap();
        assert_eq!(alias.use_count, 1);
        assert!(alias.last_used.is_some());
    }

    #[test]
    fn test_navigate_directory_not_found() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        // Create alias pointing to non-existent directory
        db.insert(Alias::new("missing", "/nonexistent/directory/path").unwrap());

        let result = navigate(&mut db, "missing");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("directory does not exist"));
    }

    #[test]
    fn test_navigate_not_a_directory() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        // Create a file and try to navigate to it
        let file = NamedTempFile::new().unwrap();
        db.insert(Alias::new("file", file.path().to_str().unwrap()).unwrap());

        let result = navigate(&mut db, "file");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));
    }

    #[test]
    fn test_navigate_fuzzy_suggestions() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        let target = tempdir().unwrap();
        db.insert(Alias::new("projects", target.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("project", target.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("work", target.path().to_str().unwrap()).unwrap());

        // Searching for "proj" - high confidence match found, prompt shown
        // In non-interactive mode, confirm() returns false, navigation cancelled
        let result = navigate(&mut db, "proj");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cancelled"), "Expected 'cancelled' error, got: {}", err);
    }

    #[test]
    fn test_navigate_no_fuzzy_matches() {
        // Test line 42: alias not found with no suggestions
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        // Add an alias that is very different from search term
        let target = tempdir().unwrap();
        db.insert(Alias::new("xyz", target.path().to_str().unwrap()).unwrap());

        // Search for something completely unrelated
        let result = navigate(&mut db, "qwerty123");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("alias 'qwerty123' not found"));
        // Should NOT contain "Did you mean" since no fuzzy matches
        assert!(!err.contains("Did you mean"));
    }

    #[test]
    fn test_navigate_single_fuzzy_match() {
        // Single high-confidence fuzzy match prompts for confirmation
        // In non-interactive mode (piped stdin), confirm() returns false
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        let target = tempdir().unwrap();
        db.insert(Alias::new("myproject", target.path().to_str().unwrap()).unwrap());

        // Typo triggers prompt - non-interactive mode declines
        let result = navigate(&mut db, "myprojet");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cancelled"), "Expected 'cancelled' error, got: {}", err);

        // Usage should NOT be recorded (user declined)
        let alias = db.get("myproject").unwrap();
        assert_eq!(alias.use_count, 0);
    }

    #[test]
    fn test_navigate_single_fuzzy_match_directory_not_found() {
        // High-confidence fuzzy match prompts for confirmation
        // In non-interactive mode, confirm() returns false before directory check
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        // Create alias pointing to non-existent directory
        db.insert(Alias::new("myproject", "/nonexistent/fuzzy/path").unwrap());

        // Typo triggers prompt - non-interactive mode declines before path check
        let result = navigate(&mut db, "myprojet");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cancelled"), "Expected 'cancelled' error, got: {}", err);
    }

    #[test]
    fn test_navigate_single_fuzzy_match_not_a_directory() {
        // High-confidence fuzzy match prompts for confirmation
        // In non-interactive mode, confirm() returns false before path check
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        // Create a file and alias to it
        let file = NamedTempFile::new().unwrap();
        db.insert(Alias::new("myproject", file.path().to_str().unwrap()).unwrap());

        // Typo triggers prompt - non-interactive mode declines before path check
        let result = navigate(&mut db, "myprojet");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cancelled"), "Expected 'cancelled' error, got: {}", err);
    }

    #[test]
    fn test_navigate_multiple_fuzzy_matches() {
        // Multiple fuzzy matches - best match (highest score >= 0.7) triggers prompt
        // In non-interactive mode, confirm() returns false
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        let target = tempdir().unwrap();
        db.insert(Alias::new("project1", target.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("project2", target.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("project3", target.path().to_str().unwrap()).unwrap());

        // "project" has high similarity to "project1" etc., prompts for best match
        let result = navigate(&mut db, "project");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cancelled"), "Expected 'cancelled' error, got: {}", err);
    }

    #[test]
    fn test_navigate_fuzzy_no_close_match() {
        // Fuzzy matches exist but none above 0.7 threshold - no prompt, just "not found"
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        let target = tempdir().unwrap();
        // Very different aliases from search term
        db.insert(Alias::new("alpha", target.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("beta", target.path().to_str().unwrap()).unwrap());

        // Search for something that has low similarity to all aliases
        let result = navigate(&mut db, "zzznothing");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // Should NOT contain "cancelled" (no prompt was shown)
        assert!(!err.contains("cancelled"), "Should not prompt for low-confidence matches");
        // Should contain "not found"
        assert!(err.contains("not found"), "Expected 'not found' error, got: {}", err);
    }

    #[test]
    fn test_completions_empty_query() {
        // completions with empty query returns all sorted
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        let target = tempdir().unwrap();
        db.insert(Alias::new("zebra", target.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("apple", target.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("mango", target.path().to_str().unwrap()).unwrap());

        // Empty query should return all aliases
        let result = completions(&db, "");
        assert!(result.is_ok());
        // The function prints to stdout, so we just verify it completes successfully
    }

    #[test]
    fn test_completions_with_query() {
        // Test lines 97-100: completions with query returns fuzzy matches
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let mut db = Database::load_from_path(&db_path).unwrap();

        let target = tempdir().unwrap();
        db.insert(Alias::new("projects", target.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("personal", target.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("work", target.path().to_str().unwrap()).unwrap());

        // Query should filter to matching aliases
        let result = completions(&db, "pro");
        assert!(result.is_ok());
    }
}
