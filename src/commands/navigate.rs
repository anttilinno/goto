//! Navigation commands: navigate, expand, completions

use std::path::Path;

use crate::alias::AliasError;
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
        } else if matches.len() == 1 {
            // Single fuzzy match - use it
            let name = matches[0].0.to_string();
            if let Some(entry) = db.get(&name) {
                // Verify directory exists
                let path = Path::new(&entry.path);
                if !path.exists() {
                    return Err(AliasError::DirectoryNotFound(entry.path.clone()).into());
                }
                if !path.is_dir() {
                    return Err(format!("not a directory: {}", entry.path).into());
                }

                let path_str = entry.path.clone();
                db.record_usage(&name)?;
                println!("{}", path_str);
                db.save()?;
            }
            Ok(())
        } else {
            // Multiple matches - show suggestions
            let suggestions: Vec<String> = matches.iter().map(|(name, _)| name.to_string()).collect();
            Err(format!(
                "alias '{}' not found. Did you mean: {}?",
                alias,
                suggestions.join(", ")
            )
            .into())
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

        // Searching for "proj" should suggest similar aliases
        let result = navigate(&mut db, "proj");
        // Should either succeed (single match) or return suggestions
        // The behavior depends on fuzzy matching threshold
        if result.is_err() {
            let err = result.unwrap_err().to_string();
            assert!(err.contains("not found") || err.contains("Did you mean"));
        }
    }
}
