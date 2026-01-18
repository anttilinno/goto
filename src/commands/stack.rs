//! Stack commands: push, pop

use std::path::Path;

use crate::alias::AliasError;
use crate::config::Config;
use crate::database::Database;
use crate::stack::Stack;

/// Push current directory to stack and navigate to alias
/// Prints the path for the shell function to cd to
pub fn push(config: &Config, db: &mut Database, alias: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Get the alias path - first check existence, then modify
    let path = {
        let entry = db.get(alias).ok_or_else(|| AliasError::NotFound(alias.to_string()))?;
        entry.path.clone()
    };

    // Verify target directory exists
    let target_path = Path::new(&path);
    if !target_path.exists() {
        return Err(AliasError::DirectoryNotFound(path).into());
    }
    if !target_path.is_dir() {
        return Err(format!("not a directory: {}", path).into());
    }

    // Get current directory
    let current = std::env::current_dir()?;

    // Push to stack (new API handles persistence automatically)
    let stack = Stack::new(config.stack_path.clone());
    stack.push(&current.to_string_lossy())?;

    // Record use after pushing to stack (so we don't record if push fails)
    if let Some(entry) = db.get_mut(alias) {
        entry.record_use();
    }
    db.save()?;

    // Print path for shell to cd to
    println!("{}", path);
    Ok(())
}

/// Pop directory from stack and return to it
/// Prints the path for the shell function to cd to
pub fn pop(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let stack = Stack::new(config.stack_path.clone());

    let path = stack.pop().map_err(|_| "stack is empty")?;

    // Verify the directory still exists
    let dir_path = Path::new(&path);
    if !dir_path.exists() {
        return Err(AliasError::DirectoryNotFound(path).into());
    }
    if !dir_path.is_dir() {
        return Err(format!("not a directory: {}", path).into());
    }

    println!("{}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::Alias;
    use crate::config::UserConfig;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_config() -> (Config, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            database_path: temp_dir.path().to_path_buf(),
            stack_path: temp_dir.path().join("goto_stack"),
            config_path: temp_dir.path().join("config.toml"),
            aliases_path: temp_dir.path().join("aliases.toml"),
            user: UserConfig::default(),
        };
        (config, temp_dir)
    }

    fn create_test_db(path: &PathBuf) -> Database {
        let mut db = Database::load_from_path(path).unwrap();
        db.insert(Alias::new("test", "/tmp").unwrap());
        db
    }

    #[test]
    fn test_push_alias_not_found() {
        let (config, _temp) = create_test_config();
        let mut db = create_test_db(&config.aliases_path);

        let result = push(&config, &mut db, "nonexistent");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"), "Expected 'not found' in error: {}", err);
    }

    #[test]
    fn test_push_directory_not_found() {
        let (config, _temp) = create_test_config();
        let mut db = Database::load_from_path(&config.aliases_path).unwrap();
        db.insert(Alias::new("missing", "/nonexistent/path/that/does/not/exist").unwrap());

        let result = push(&config, &mut db, "missing");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not exist") || err.contains("not found"),
                "Expected directory error in: {}", err);
    }

    #[test]
    fn test_push_not_a_directory() {
        let (config, temp) = create_test_config();

        // Create a file (not a directory) to point the alias at
        let file_path = temp.path().join("not_a_dir");
        fs::write(&file_path, "test").unwrap();

        let mut db = Database::load_from_path(&config.aliases_path).unwrap();
        db.insert(Alias::new("file", file_path.to_string_lossy().as_ref()).unwrap());

        let result = push(&config, &mut db, "file");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not a directory"), "Expected 'not a directory' in: {}", err);
    }

    #[test]
    fn test_pop_empty_stack() {
        let (config, _temp) = create_test_config();

        let result = pop(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("empty"), "Expected 'empty' in error: {}", err);
    }

    #[test]
    fn test_pop_directory_not_found() {
        let (config, temp) = create_test_config();

        // Create a directory, push it to the stack, then remove it
        let dir_path = temp.path().join("will_be_deleted");
        fs::create_dir(&dir_path).unwrap();

        let stack = Stack::new(config.stack_path.clone());
        stack.push(dir_path.to_string_lossy().as_ref()).unwrap();

        // Remove the directory
        fs::remove_dir(&dir_path).unwrap();

        let result = pop(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not exist") || err.contains("not found"),
                "Expected directory error in: {}", err);
    }

    #[test]
    fn test_push_and_pop() {
        let (config, _temp) = create_test_config();
        let mut db = create_test_db(&config.aliases_path);

        // Push should succeed (alias points to /tmp which exists)
        let result = push(&config, &mut db, "test");
        assert!(result.is_ok());

        // Pop should succeed and return the pushed directory
        let result = pop(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_push_records_usage() {
        let (config, _temp) = create_test_config();
        let mut db = create_test_db(&config.aliases_path);

        // Initial use count should be 0
        assert_eq!(db.get("test").unwrap().use_count, 0);

        // Push should record usage
        let result = push(&config, &mut db, "test");
        assert!(result.is_ok());

        // Use count should be incremented
        assert_eq!(db.get("test").unwrap().use_count, 1);
        assert!(db.get("test").unwrap().last_used.is_some());
    }

    #[test]
    fn test_push_saves_current_directory() {
        let (config, _temp) = create_test_config();
        let mut db = create_test_db(&config.aliases_path);

        // Get the current working directory
        let cwd = std::env::current_dir().unwrap();

        // Push should succeed
        let result = push(&config, &mut db, "test");
        assert!(result.is_ok());

        // Check that the current directory was pushed to the stack
        let stack = Stack::new(config.stack_path.clone());
        let popped = stack.pop().unwrap();
        assert_eq!(popped, cwd.to_string_lossy());
    }

    #[test]
    fn test_push_pop_multiple() {
        let (config, temp) = create_test_config();

        // Create two directories to use as aliases
        let dir1 = temp.path().join("dir1");
        let dir2 = temp.path().join("dir2");
        fs::create_dir(&dir1).unwrap();
        fs::create_dir(&dir2).unwrap();

        let mut db = Database::load_from_path(&config.aliases_path).unwrap();
        db.insert(Alias::new("alias1", dir1.to_string_lossy().as_ref()).unwrap());
        db.insert(Alias::new("alias2", dir2.to_string_lossy().as_ref()).unwrap());

        // Push twice
        push(&config, &mut db, "alias1").unwrap();
        push(&config, &mut db, "alias2").unwrap();

        // Stack should have 2 entries
        let stack = Stack::new(config.stack_path.clone());
        assert_eq!(stack.size().unwrap(), 2);

        // Pop should work twice (directories exist because they're the cwd copies)
        // Since we pushed the current working directory twice, both pops should succeed
        let result1 = pop(&config);
        assert!(result1.is_ok());

        let result2 = pop(&config);
        assert!(result2.is_ok());

        // Third pop should fail (empty stack)
        let result3 = pop(&config);
        assert!(result3.is_err());
    }
}
