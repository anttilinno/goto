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

/// Rename or merge a tag across all aliases
///
/// If target tag doesn't exist: simple rename
/// If target tag exists: merge (aliases with old tag gain new tag, old tag removed)
///
/// # Arguments
/// * `db` - The alias database
/// * `config` - Config for table styling
/// * `old_tag` - The tag to rename/remove
/// * `new_tag` - The target tag name
/// * `dry_run` - If true, only preview changes without modifying
/// * `force` - If true, skip confirmation prompt
pub fn rename_tag(
    db: &mut Database,
    config: &Config,
    old_tag: &str,
    new_tag: &str,
    dry_run: bool,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Normalize both tags
    let old_tag = old_tag.trim().to_lowercase();
    let new_tag = new_tag.trim().to_lowercase();

    // Validate new tag
    validate_tag(&new_tag)?;

    // Check if old_tag exists
    let all_tags = db.get_all_tags();
    if !all_tags.contains_key(&old_tag) {
        return Err(format!("tag '{}' not found", old_tag).into());
    }

    // Find affected aliases
    let affected: Vec<String> = db
        .all()
        .filter(|a| a.has_tag(&old_tag))
        .map(|a| a.name.clone())
        .collect();

    if affected.is_empty() {
        println!("No aliases with tag '{}'", old_tag);
        return Ok(());
    }

    // Determine if this is a merge (new_tag already exists)
    let is_merge = all_tags.contains_key(&new_tag);
    let operation = if is_merge { "merge" } else { "rename" };

    // Confirmation prompt (unless force or dry_run)
    if !force && !dry_run {
        let message = format!(
            "Will {} tag '{}' to '{}' affecting {} alias{}",
            operation,
            old_tag,
            new_tag,
            affected.len(),
            if affected.len() == 1 { "" } else { "es" }
        );
        if !confirm(&message, false)? {
            return Err("Tag rename cancelled".into());
        }
    }

    // Dry run: display preview table
    if dry_run {
        println!(
            "Would {} tag '{}' to '{}' affecting {} alias{} (dry-run):",
            operation,
            old_tag,
            new_tag,
            affected.len(),
            if affected.len() == 1 { "" } else { "es" }
        );

        let style = TableStyle::from(config.user.display.table_style.as_str());
        let mut table = create_table(style);
        table.set_header(vec!["Name", "Current Tags", "After"]);

        for name in &affected {
            if let Some(alias) = db.get(name) {
                let current_tags = alias.tags.join(", ");

                // Compute "after" tags: remove old, add new if not present
                let mut after_tags: Vec<String> = alias
                    .tags
                    .iter()
                    .filter(|t| *t != &old_tag)
                    .cloned()
                    .collect();
                if !after_tags.contains(&new_tag) {
                    after_tags.push(new_tag.clone());
                }
                after_tags.sort();
                let after = after_tags.join(", ");

                table.add_row(vec![name.clone(), current_tags, after]);
            }
        }

        println!("{}", table);
        return Ok(());
    }

    // Apply changes atomically
    for name in &affected {
        if let Some(alias) = db.get_mut(name) {
            alias.remove_tag(&old_tag);
            if !alias.has_tag(&new_tag) {
                alias.add_tag(&new_tag);
            }
        }
    }

    // Single save at end
    db.save()?;

    println!(
        "{}d tag '{}' to '{}' on {} alias{}",
        if is_merge { "Merge" } else { "Rename" },
        old_tag,
        new_tag,
        affected.len(),
        if affected.len() == 1 { "" } else { "es" }
    );

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

    // Tests for rename_tag function

    #[test]
    fn test_rename_tag_basic() {
        let (mut db, _file) = create_test_db_with_multiple_aliases();
        let config = Config::load().unwrap();

        // Add "work" tag to proj1 and proj2
        tag(&mut db, "proj1", "work", true).unwrap();
        tag(&mut db, "proj2", "work", true).unwrap();

        // Rename "work" to "job" with force (target doesn't exist)
        let result = rename_tag(&mut db, &config, "work", "job", false, true);
        assert!(result.is_ok());

        // Verify: "work" tag gone, "job" tag exists
        assert!(!db.get_all_tags().contains_key("work"));
        assert!(db.get_all_tags().contains_key("job"));
        assert!(db.get("proj1").unwrap().has_tag("job"));
        assert!(db.get("proj2").unwrap().has_tag("job"));
    }

    #[test]
    fn test_rename_tag_merge() {
        let (mut db, _file) = create_test_db_with_multiple_aliases();
        let config = Config::load().unwrap();

        // Add "work" to proj1, "job" to proj2, and "job" to docs
        tag(&mut db, "proj1", "work", true).unwrap();
        tag(&mut db, "proj2", "job", true).unwrap();
        tag(&mut db, "docs", "job", true).unwrap();

        // Also add "job" to proj1 (so we can verify no duplicate)
        tag(&mut db, "proj1", "job", true).unwrap();

        // Rename/merge "work" into "job"
        let result = rename_tag(&mut db, &config, "work", "job", false, true);
        assert!(result.is_ok());

        // Verify: "work" tag gone
        assert!(!db.get_all_tags().contains_key("work"));

        // proj1 should have "job" only once
        let proj1 = db.get("proj1").unwrap();
        assert!(proj1.has_tag("job"));
        assert_eq!(proj1.tags.iter().filter(|t| *t == "job").count(), 1);
    }

    #[test]
    fn test_rename_tag_source_not_found() {
        let (mut db, _file) = create_test_db();
        let config = Config::load().unwrap();

        let result = rename_tag(&mut db, &config, "nonexistent", "newtag", false, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_rename_tag_normalizes_case() {
        let (mut db, _file) = create_test_db();
        let config = Config::load().unwrap();

        // Add "work" tag
        tag(&mut db, "test", "work", true).unwrap();

        // Rename "WORK" to "JOB" - should normalize to lowercase
        let result = rename_tag(&mut db, &config, "WORK", "JOB", false, true);
        assert!(result.is_ok());

        // Verify lowercase "job" exists
        assert!(db.get_all_tags().contains_key("job"));
        assert!(db.get("test").unwrap().has_tag("job"));
    }

    #[test]
    fn test_rename_tag_force_bypasses_confirm() {
        let (mut db, _file) = create_test_db();
        let config = Config::load().unwrap();

        // Add tag
        tag(&mut db, "test", "work", true).unwrap();

        // Without force, should fail in non-interactive (confirm returns false)
        let result = rename_tag(&mut db, &config, "work", "job", false, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));

        // With force, should succeed
        let result = rename_tag(&mut db, &config, "work", "job", false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rename_tag_dry_run_no_changes() {
        let (mut db, _file) = create_test_db();
        let config = Config::load().unwrap();

        // Add tag
        tag(&mut db, "test", "work", true).unwrap();

        // Dry run should not make changes
        let result = rename_tag(&mut db, &config, "work", "job", true, false);
        assert!(result.is_ok());

        // Verify original tag still exists
        assert!(db.get_all_tags().contains_key("work"));
        assert!(!db.get_all_tags().contains_key("job"));
        assert!(db.get("test").unwrap().has_tag("work"));
    }
}
