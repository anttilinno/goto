//! Cleanup commands

use std::path::Path;

use crate::database::Database;

/// Remove aliases with invalid (non-existent) paths
pub fn cleanup(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    let invalid: Vec<String> = db
        .all()
        .filter(|a| !Path::new(&a.path).exists())
        .map(|a| a.name.clone())
        .collect();

    if invalid.is_empty() {
        println!("All aliases point to valid paths.");
        return Ok(());
    }

    println!("Removing {} aliases with invalid paths:", invalid.len());
    for name in &invalid {
        if let Some(alias) = db.get(name) {
            println!("  {} -> {} (path does not exist)", name, alias.path);
        }
        db.remove(name);
    }

    db.save()?;
    println!("Cleanup complete.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::Alias;
    use tempfile::{NamedTempFile, TempDir};

    fn create_test_db() -> (Database, NamedTempFile) {
        let file = NamedTempFile::new().unwrap();
        let db = Database::load_from_path(file.path()).unwrap();
        (db, file)
    }

    #[test]
    fn test_cleanup_all_valid() {
        let (mut db, _file) = create_test_db();
        let temp_dir = TempDir::new().unwrap();

        db.insert(Alias::new("valid", temp_dir.path().to_str().unwrap()).unwrap());

        let result = cleanup(&mut db);
        assert!(result.is_ok());
        assert!(db.contains("valid"));
    }

    #[test]
    fn test_cleanup_removes_invalid() {
        let (mut db, _file) = create_test_db();
        let temp_dir = TempDir::new().unwrap();

        db.insert(Alias::new("valid", temp_dir.path().to_str().unwrap()).unwrap());
        db.insert(Alias::new("invalid", "/nonexistent/path/12345").unwrap());

        let result = cleanup(&mut db);
        assert!(result.is_ok());
        assert!(db.contains("valid"));
        assert!(!db.contains("invalid"));
    }

    #[test]
    fn test_cleanup_empty() {
        let (mut db, _file) = create_test_db();
        let result = cleanup(&mut db);
        assert!(result.is_ok());
    }
}
