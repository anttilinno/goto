//! Tag commands: tag, untag, list_tags

use crate::alias::validate_tag;
use crate::config::Config;
use crate::confirm;
use crate::database::Database;
use crate::table::{create_table, TableStyle};

/// Add a tag to an alias
///
/// Validates and normalizes the tag to lowercase before adding.
/// This operation is idempotent - adding an existing tag is a no-op.
///
/// # Arguments
/// * `db` - The alias database
/// * `alias` - The alias to tag
/// * `tag_name` - The tag to add
/// * `force` - If true, skip confirmation for new tags
pub fn tag(db: &mut Database, alias: &str, tag_name: &str, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Normalize and validate the tag
    let tag_name = tag_name.trim().to_lowercase();
    validate_tag(&tag_name)?;

    // Check if this is a new tag (doesn't exist on any alias)
    let existing_tags = db.get_all_tags();
    let is_new_tag = !existing_tags.contains_key(&tag_name);
    let has_any_tags = !existing_tags.is_empty();

    // Confirm new tag creation if:
    // - Tag doesn't exist anywhere
    // - Other tags exist (not bootstrapping)
    // - Not using --force
    if is_new_tag && has_any_tags && !force {
        let message = format!("Tag '{}' doesn't exist. Create it?", tag_name);
        if !confirm(&message, false)? {
            return Err("Tag creation cancelled".into());
        }
    }

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

        // First tag created without confirmation (bootstrapping)
        let result = tag(&mut db, "test", "work", false);
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(alias.has_tag("work"));
    }

    #[test]
    fn test_tag_normalizes_to_lowercase() {
        let (mut db, _file) = create_test_db();

        // First tag - no confirmation needed
        let result = tag(&mut db, "test", "WORK", false);
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(alias.has_tag("work"));
        assert!(!alias.has_tag("WORK"));
    }

    #[test]
    fn test_tag_trims_whitespace() {
        let (mut db, _file) = create_test_db();

        // First tag - no confirmation needed
        let result = tag(&mut db, "test", "  work  ", false);
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(alias.has_tag("work"));
    }

    #[test]
    fn test_tag_validates_format() {
        let (mut db, _file) = create_test_db();

        // Empty tag should fail
        let result = tag(&mut db, "test", "", true);
        assert!(result.is_err());

        // Invalid characters should fail
        let result = tag(&mut db, "test", "work@home", true);
        assert!(result.is_err());

        // Starting with hyphen should fail
        let result = tag(&mut db, "test", "-work", true);
        assert!(result.is_err());
    }

    #[test]
    fn test_tag_idempotent() {
        let (mut db, _file) = create_test_db();

        // Add tag twice - first one succeeds (bootstrapping), second is idempotent (tag exists)
        tag(&mut db, "test", "work", false).unwrap();
        let result = tag(&mut db, "test", "work", false);
        assert!(result.is_ok());

        // Tag should still only appear once
        let alias = db.get("test").unwrap();
        assert_eq!(alias.tags.iter().filter(|t| *t == "work").count(), 1);
    }

    #[test]
    fn test_tag_not_found() {
        let (mut db, _file) = create_test_db();
        // First tag - no confirmation needed, but alias doesn't exist
        let result = tag(&mut db, "nonexistent", "work", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_untag() {
        let (mut db, _file) = create_test_db();
        tag(&mut db, "test", "work", true).unwrap();

        let result = untag(&mut db, "test", "work");
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(!alias.has_tag("work"));
    }

    #[test]
    fn test_untag_normalizes_to_lowercase() {
        let (mut db, _file) = create_test_db();
        tag(&mut db, "test", "work", true).unwrap();

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
        // Use force=true for second tag (first tag exists)
        tag(&mut db, "test", "work", true).unwrap();
        tag(&mut db, "test", "important", true).unwrap();

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

        // Add "work" tag to two aliases (use force=true for subsequent new tags)
        tag(&mut db, "proj1", "work", true).unwrap();
        tag(&mut db, "proj2", "work", true).unwrap();

        // Add "docs" tag to one alias
        tag(&mut db, "docs", "docs", true).unwrap();

        let tag_counts = db.get_all_tags();
        assert_eq!(tag_counts.get("work"), Some(&2));
        assert_eq!(tag_counts.get("docs"), Some(&1));
    }

    #[test]
    fn test_list_tags_raw() {
        let (mut db, _file) = create_test_db();
        tag(&mut db, "test", "work", true).unwrap();
        tag(&mut db, "test", "important", true).unwrap();

        let result = list_tags_raw(&db);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_tags_raw_empty() {
        let (db, _file) = create_test_db();
        let result = list_tags_raw(&db);
        assert!(result.is_ok());
    }

    // Tests for confirmation behavior (TAG-01 through TAG-04)

    #[test]
    fn test_tag_first_tag_no_confirmation_needed() {
        // TAG-02: First tag is created silently without prompt
        let (mut db, _file) = create_test_db();

        // No tags exist, so first tag should succeed without confirmation
        let result = tag(&mut db, "test", "work", false);
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(alias.has_tag("work"));
    }

    #[test]
    fn test_tag_new_tag_denied_in_non_interactive() {
        // TAG-03: Non-interactive mode (piped stdin) denies new tag creation
        let (mut db, _file) = create_test_db();

        // Create first tag (bootstrapping - succeeds)
        tag(&mut db, "test", "existing", true).unwrap();

        // Try to create new tag without force - should be denied in non-interactive
        // (tests run with piped stdin, so confirm() returns default=false)
        let result = tag(&mut db, "test", "newtag", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }

    #[test]
    fn test_tag_force_bypasses_confirmation() {
        // TAG-04: --force bypasses all tag confirmation prompts
        let (mut db, _file) = create_test_db();

        // Create first tag
        tag(&mut db, "test", "existing", true).unwrap();

        // With force=true, new tag creation should succeed
        let result = tag(&mut db, "test", "newtag", true);
        assert!(result.is_ok());

        let alias = db.get("test").unwrap();
        assert!(alias.has_tag("newtag"));
    }

    #[test]
    fn test_tag_existing_tag_no_confirmation() {
        // Adding a tag that already exists on another alias needs no confirmation
        let (mut db, _file) = create_test_db_with_multiple_aliases();

        // Create tag on proj1
        tag(&mut db, "proj1", "work", true).unwrap();

        // Add same tag to proj2 - should succeed without confirmation (tag exists)
        let result = tag(&mut db, "proj2", "work", false);
        assert!(result.is_ok());

        let alias = db.get("proj2").unwrap();
        assert!(alias.has_tag("work"));
    }
}
