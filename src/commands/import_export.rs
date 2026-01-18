//! Import and export commands

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::alias::{validate_alias, Alias};
use crate::database::Database;

/// Export aliases as TOML to stdout
pub fn export(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    if db.is_empty() {
        eprintln!("No aliases to export");
        return Ok(());
    }

    let toml = db.export_toml()?;
    print!("{}", toml);
    Ok(())
}

/// Import result statistics
#[derive(Debug, Default)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub renamed: usize,
    pub warnings: Vec<String>,
}

/// Import strategy for handling conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImportStrategy {
    #[default]
    Skip,      // Skip existing aliases
    Overwrite, // Overwrite existing aliases
    Rename,    // Rename conflicting aliases with suffix
}

impl ImportStrategy {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "skip" => Ok(ImportStrategy::Skip),
            "overwrite" => Ok(ImportStrategy::Overwrite),
            "rename" => Ok(ImportStrategy::Rename),
            _ => Err(format!(
                "invalid strategy: {} (must be skip, overwrite, or rename)",
                s
            )),
        }
    }
}

/// Import aliases from a TOML file with the specified strategy
pub fn import(
    db: &mut Database,
    file_path: &str,
    strategy: ImportStrategy,
) -> Result<ImportResult, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let result = import_from_content(db, &content, strategy)?;
    db.save()?;
    Ok(result)
}

/// Import aliases from TOML content string with the specified strategy
pub fn import_from_content(
    db: &mut Database,
    content: &str,
    strategy: ImportStrategy,
) -> Result<ImportResult, Box<dyn std::error::Error>> {
    // Parse TOML content to get aliases
    #[derive(serde::Deserialize)]
    struct ImportFile {
        #[serde(default)]
        aliases: Vec<Alias>,
    }

    let import_data: ImportFile = toml::from_str(content)?;

    if import_data.aliases.is_empty() {
        return Err("no aliases found in import file".into());
    }

    // Build map of existing alias names for quick lookup
    let mut existing_names: HashMap<String, bool> = db.names().map(|n| (n.to_string(), true)).collect();

    let mut result = ImportResult::default();

    for import_alias in import_data.aliases {
        // Validate alias name
        if let Err(e) = validate_alias(&import_alias.name) {
            result.warnings.push(format!(
                "skipping invalid alias name '{}': {}",
                import_alias.name, e
            ));
            result.skipped += 1;
            continue;
        }

        // Check if path exists (warn but don't skip)
        if !Path::new(&import_alias.path).exists() {
            result.warnings.push(format!(
                "warning: path does not exist for alias '{}': {}",
                import_alias.name, import_alias.path
            ));
        }

        if existing_names.contains_key(&import_alias.name) {
            // Alias already exists - handle based on strategy
            match strategy {
                ImportStrategy::Skip => {
                    result.skipped += 1;
                }
                ImportStrategy::Overwrite => {
                    db.insert(import_alias);
                    result.imported += 1;
                }
                ImportStrategy::Rename => {
                    let new_name = find_unique_name(&import_alias.name, &existing_names);
                    let mut renamed_alias = import_alias;
                    renamed_alias.name = new_name.clone();
                    existing_names.insert(new_name, true);
                    db.insert(renamed_alias);
                    result.renamed += 1;
                }
            }
        } else {
            // New alias - add it
            existing_names.insert(import_alias.name.clone(), true);
            db.insert(import_alias);
            result.imported += 1;
        }
    }

    Ok(result)
}

/// Generate a unique alias name by appending a numeric suffix
fn find_unique_name(base_name: &str, existing_names: &HashMap<String, bool>) -> String {
    let mut suffix = 2;
    loop {
        let new_name = format!("{}_{}", base_name, suffix);
        if !existing_names.contains_key(&new_name) {
            return new_name;
        }
        suffix += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::Alias;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    fn create_test_db() -> (Database, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("aliases");
        let db = Database::load_from_path(&path).unwrap();
        (db, dir)
    }

    fn create_test_db_with_alias() -> (Database, tempfile::TempDir) {
        let (mut db, dir) = create_test_db();
        db.insert(Alias::new("test", "/tmp").unwrap());
        (db, dir)
    }

    #[test]
    fn test_export_empty_database() {
        let (db, _dir) = create_test_db();
        // Export should succeed but print message to stderr
        let result = export(&db);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_with_aliases() {
        let (mut db, _dir) = create_test_db();
        let mut alias = Alias::new("test", "/tmp/test").unwrap();
        alias.add_tag("work");
        alias.use_count = 5;
        db.insert(alias);

        let result = export(&db);
        assert!(result.is_ok());
    }

    #[test]
    fn test_import_strategy_from_str() {
        assert_eq!(ImportStrategy::from_str("skip").unwrap(), ImportStrategy::Skip);
        assert_eq!(ImportStrategy::from_str("SKIP").unwrap(), ImportStrategy::Skip);
        assert_eq!(ImportStrategy::from_str("overwrite").unwrap(), ImportStrategy::Overwrite);
        assert_eq!(ImportStrategy::from_str("rename").unwrap(), ImportStrategy::Rename);
        assert!(ImportStrategy::from_str("invalid").is_err());
    }

    #[test]
    fn test_import_new_aliases() {
        let (mut db, _dir) = create_test_db();

        let mut import_file = NamedTempFile::new().unwrap();
        writeln!(
            import_file,
            r#"[[aliases]]
name = "imported"
path = "/tmp"
tags = ["work"]
use_count = 10
created_at = "2024-01-01T00:00:00Z"
"#
        )
        .unwrap();

        let result = import(&mut db, import_file.path().to_str().unwrap(), ImportStrategy::Skip).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.renamed, 0);
        assert!(db.contains("imported"));

        let alias = db.get("imported").unwrap();
        assert_eq!(alias.use_count, 10);
        assert!(alias.has_tag("work"));
    }

    #[test]
    fn test_import_skip_existing() {
        let (mut db, _dir) = create_test_db_with_alias();

        let mut import_file = NamedTempFile::new().unwrap();
        writeln!(
            import_file,
            r#"[[aliases]]
name = "test"
path = "/different/path"
tags = []
use_count = 0
created_at = "2024-01-01T00:00:00Z"

[[aliases]]
name = "new"
path = "/tmp/new"
tags = []
use_count = 0
created_at = "2024-01-01T00:00:00Z"
"#
        )
        .unwrap();

        let result = import(&mut db, import_file.path().to_str().unwrap(), ImportStrategy::Skip).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.renamed, 0);

        // Original "test" should be unchanged
        assert_eq!(db.get("test").unwrap().path, "/tmp");
        // New alias should be added
        assert!(db.contains("new"));
    }

    #[test]
    fn test_import_overwrite_existing() {
        let (mut db, _dir) = create_test_db_with_alias();

        let mut import_file = NamedTempFile::new().unwrap();
        writeln!(
            import_file,
            r#"[[aliases]]
name = "test"
path = "/different/path"
tags = ["imported"]
use_count = 99
created_at = "2024-01-01T00:00:00Z"
"#
        )
        .unwrap();

        let result = import(&mut db, import_file.path().to_str().unwrap(), ImportStrategy::Overwrite).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.renamed, 0);

        // "test" should be overwritten
        let alias = db.get("test").unwrap();
        assert_eq!(alias.path, "/different/path");
        assert_eq!(alias.use_count, 99);
        assert!(alias.has_tag("imported"));
    }

    #[test]
    fn test_import_rename_existing() {
        let (mut db, _dir) = create_test_db_with_alias();

        let mut import_file = NamedTempFile::new().unwrap();
        writeln!(
            import_file,
            r#"[[aliases]]
name = "test"
path = "/different/path"
tags = []
use_count = 0
created_at = "2024-01-01T00:00:00Z"
"#
        )
        .unwrap();

        let result = import(&mut db, import_file.path().to_str().unwrap(), ImportStrategy::Rename).unwrap();
        assert_eq!(result.imported, 0);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.renamed, 1);

        // Original "test" should be unchanged
        assert_eq!(db.get("test").unwrap().path, "/tmp");
        // Renamed alias should exist
        assert!(db.contains("test_2"));
        assert_eq!(db.get("test_2").unwrap().path, "/different/path");
    }

    #[test]
    fn test_import_rename_multiple_conflicts() {
        let (mut db, _dir) = create_test_db();
        db.insert(Alias::new("proj", "/tmp/proj").unwrap());
        db.insert(Alias::new("proj_2", "/tmp/proj2").unwrap());

        let mut import_file = NamedTempFile::new().unwrap();
        writeln!(
            import_file,
            r#"[[aliases]]
name = "proj"
path = "/new/proj"
tags = []
use_count = 0
created_at = "2024-01-01T00:00:00Z"
"#
        )
        .unwrap();

        let result = import(&mut db, import_file.path().to_str().unwrap(), ImportStrategy::Rename).unwrap();
        assert_eq!(result.renamed, 1);

        // Should skip to proj_3 since proj_2 exists
        assert!(db.contains("proj_3"));
    }

    #[test]
    fn test_import_invalid_alias_name() {
        let (mut db, _dir) = create_test_db();

        let mut import_file = NamedTempFile::new().unwrap();
        writeln!(
            import_file,
            r#"[[aliases]]
name = "-invalid"
path = "/tmp"
tags = []
use_count = 0
created_at = "2024-01-01T00:00:00Z"

[[aliases]]
name = "valid"
path = "/tmp"
tags = []
use_count = 0
created_at = "2024-01-01T00:00:00Z"
"#
        )
        .unwrap();

        let result = import(&mut db, import_file.path().to_str().unwrap(), ImportStrategy::Skip).unwrap();
        assert_eq!(result.imported, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("invalid alias name"));

        assert!(!db.contains("-invalid"));
        assert!(db.contains("valid"));
    }

    #[test]
    fn test_import_warns_nonexistent_path() {
        let (mut db, _dir) = create_test_db();

        let mut import_file = NamedTempFile::new().unwrap();
        writeln!(
            import_file,
            r#"[[aliases]]
name = "missing"
path = "/nonexistent/path/that/does/not/exist"
tags = []
use_count = 0
created_at = "2024-01-01T00:00:00Z"
"#
        )
        .unwrap();

        let result = import(&mut db, import_file.path().to_str().unwrap(), ImportStrategy::Skip).unwrap();
        assert_eq!(result.imported, 1);
        assert!(result.warnings.iter().any(|w| w.contains("path does not exist")));

        // Alias should still be imported despite warning
        assert!(db.contains("missing"));
    }

    #[test]
    fn test_import_empty_file() {
        let (mut db, _dir) = create_test_db();

        let mut import_file = NamedTempFile::new().unwrap();
        writeln!(import_file, "").unwrap();

        let result = import(&mut db, import_file.path().to_str().unwrap(), ImportStrategy::Skip);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no aliases found"));
    }

    #[test]
    fn test_import_file_not_found() {
        let (mut db, _dir) = create_test_db();
        let result = import(&mut db, "/nonexistent/file.toml", ImportStrategy::Skip);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_unique_name() {
        let mut existing: HashMap<String, bool> = HashMap::new();
        existing.insert("test".to_string(), true);

        assert_eq!(find_unique_name("test", &existing), "test_2");

        existing.insert("test_2".to_string(), true);
        assert_eq!(find_unique_name("test", &existing), "test_3");

        existing.insert("test_3".to_string(), true);
        existing.insert("test_4".to_string(), true);
        assert_eq!(find_unique_name("test", &existing), "test_5");
    }

    #[test]
    fn test_import_preserves_metadata() {
        let (mut db, _dir) = create_test_db();

        let mut import_file = NamedTempFile::new().unwrap();
        writeln!(
            import_file,
            r#"[[aliases]]
name = "imported"
path = "/tmp"
tags = ["work", "important"]
use_count = 42
created_at = "2024-06-15T10:30:00Z"
"#
        )
        .unwrap();

        import(&mut db, import_file.path().to_str().unwrap(), ImportStrategy::Skip).unwrap();

        let alias = db.get("imported").unwrap();
        assert_eq!(alias.use_count, 42);
        assert!(alias.has_tag("work"));
        assert!(alias.has_tag("important"));
    }
}
