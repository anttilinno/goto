//! Tag commands: tag, untag, list_tags

use crate::alias::validate_tag;
use crate::config::Config;
use crate::database::Database;
use crate::table::{create_table, TableStyle};

/// Add a tag to an alias
///
/// Validates and normalizes the tag to lowercase before adding.
/// This operation is idempotent - adding an existing tag is a no-op.
pub fn tag(db: &mut Database, alias: &str, tag_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Normalize and validate the tag
    let tag_name = tag_name.trim().to_lowercase();
    validate_tag(&tag_name)?;

    if let Some(entry) = db.get_mut(alias) {
        entry.add_tag(&tag_name);
        db.save()?;
        println!("Added tag '{}' to alias '{}'", tag_name, alias);
        Ok(())
    } else {
        Err(format!("alias '{}' not found", alias).into())
    }
}

/// Remove a tag from an alias
///
/// This operation is idempotent - removing a non-existent tag is a no-op.
pub fn untag(db: &mut Database, alias: &str, tag_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tag_name = tag_name.trim().to_lowercase();

    if let Some(entry) = db.get_mut(alias) {
        if entry.remove_tag(&tag_name) {
            db.save()?;
            println!("Removed tag '{}' from alias '{}'", tag_name, alias);
        } else {
            println!("Removed tag '{}' from alias '{}'", tag_name, alias);
        }
        Ok(())
    } else {
        Err(format!("alias '{}' not found", alias).into())
    }
}

/// List all unique tags with their counts
pub fn list_tags(db: &Database, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let tag_counts = db.get_all_tags();

    if tag_counts.is_empty() {
        println!("No tags found");
        return Ok(());
    }

    // Sort tags alphabetically
    let mut tags: Vec<_> = tag_counts.into_iter().collect();
    tags.sort_by(|a, b| a.0.cmp(&b.0));

    let style = TableStyle::from(config.user.display.table_style.as_str());
    let mut table = create_table(style);
    table.set_header(vec!["Tag", "Aliases"]);

    for (tag, count) in tags {
        let plural = if count == 1 { "alias" } else { "aliases" };
        table.add_row(vec![tag, format!("{} {}", count, plural)]);
    }

    println!("{}", table);
    Ok(())
}

/// List tag names only (for shell completion)
pub fn list_tags_raw(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let tag_counts = db.get_all_tags();

    // Sort tags alphabetically
    let mut tags: Vec<_> = tag_counts.keys().cloned().collect();
    tags.sort();

    for tag in tags {
        println!("{}", tag);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::Alias;
    use crate::config::Config;
    use tempfile::NamedTempFile;

    fn create_test_db() -> (Database, NamedTempFile) {
        let file = NamedTempFile::new().unwrap();
        let mut db = Database::load_from_path(file.path()).unwrap();
        db.insert(Alias::new("test", "/tmp").unwrap());
        (db, file)
    }

    fn create_test_db_with_multiple_aliases() -> (Database, NamedTempFile) {
        let file = NamedTempFile::new().unwrap();
        let mut db = Database::load_from_path(file.path()).unwrap();
        db.insert(Alias::new("proj1", "/tmp/proj1").unwrap());
        db.insert(Alias::new("proj2", "/tmp/proj2").unwrap());
        db.insert(Alias::new("docs", "/tmp/docs").unwrap());
        (db, file)
    }

    #[test]
    fn test_tag() {
        let (mut db, _file) = create_test_db();

        let result = tag(&mut db, "test", "work");
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(alias.has_tag("work"));
    }

    #[test]
    fn test_tag_normalizes_to_lowercase() {
        let (mut db, _file) = create_test_db();

        let result = tag(&mut db, "test", "WORK");
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(alias.has_tag("work"));
        assert!(!alias.has_tag("WORK"));
    }

    #[test]
    fn test_tag_trims_whitespace() {
        let (mut db, _file) = create_test_db();

        let result = tag(&mut db, "test", "  work  ");
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(alias.has_tag("work"));
    }

    #[test]
    fn test_tag_validates_format() {
        let (mut db, _file) = create_test_db();

        // Empty tag should fail
        let result = tag(&mut db, "test", "");
        assert!(result.is_err());

        // Invalid characters should fail
        let result = tag(&mut db, "test", "work@home");
        assert!(result.is_err());

        // Starting with hyphen should fail
        let result = tag(&mut db, "test", "-work");
        assert!(result.is_err());
    }

    #[test]
    fn test_tag_idempotent() {
        let (mut db, _file) = create_test_db();

        // Add tag twice
        tag(&mut db, "test", "work").unwrap();
        let result = tag(&mut db, "test", "work");
        assert!(result.is_ok());

        // Tag should still only appear once
        let alias = db.get("test").unwrap();
        assert_eq!(alias.tags.iter().filter(|t| *t == "work").count(), 1);
    }

    #[test]
    fn test_tag_not_found() {
        let (mut db, _file) = create_test_db();
        let result = tag(&mut db, "nonexistent", "work");
        assert!(result.is_err());
    }

    #[test]
    fn test_untag() {
        let (mut db, _file) = create_test_db();
        tag(&mut db, "test", "work").unwrap();

        let result = untag(&mut db, "test", "work");
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(!alias.has_tag("work"));
    }

    #[test]
    fn test_untag_normalizes_to_lowercase() {
        let (mut db, _file) = create_test_db();
        tag(&mut db, "test", "work").unwrap();

        // Should remove "work" even when passed as "WORK"
        let result = untag(&mut db, "test", "WORK");
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(!alias.has_tag("work"));
    }

    #[test]
    fn test_untag_idempotent() {
        let (mut db, _file) = create_test_db();

        // Removing a non-existent tag should succeed (idempotent)
        let result = untag(&mut db, "test", "nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_untag_alias_not_found() {
        let (mut db, _file) = create_test_db();
        let result = untag(&mut db, "nonexistent", "work");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_tags() {
        let (mut db, _file) = create_test_db();
        let config = Config::load().unwrap();
        tag(&mut db, "test", "work").unwrap();
        tag(&mut db, "test", "important").unwrap();

        let result = list_tags(&db, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_tags_empty() {
        let (db, _file) = create_test_db();
        let config = Config::load().unwrap();
        let result = list_tags(&db, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_tags_shows_counts() {
        let (mut db, _file) = create_test_db_with_multiple_aliases();

        // Add "work" tag to two aliases
        tag(&mut db, "proj1", "work").unwrap();
        tag(&mut db, "proj2", "work").unwrap();

        // Add "docs" tag to one alias
        tag(&mut db, "docs", "docs").unwrap();

        let tag_counts = db.get_all_tags();
        assert_eq!(tag_counts.get("work"), Some(&2));
        assert_eq!(tag_counts.get("docs"), Some(&1));
    }

    #[test]
    fn test_list_tags_raw() {
        let (mut db, _file) = create_test_db();
        tag(&mut db, "test", "work").unwrap();
        tag(&mut db, "test", "important").unwrap();

        let result = list_tags_raw(&db);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_tags_raw_empty() {
        let (db, _file) = create_test_db();
        let result = list_tags_raw(&db);
        assert!(result.is_ok());
    }
}
