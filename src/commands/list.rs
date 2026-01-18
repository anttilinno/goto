//! List commands: list, list_with_options, list_names

use std::io::{self, Write};

use crate::config::Config;
use crate::database::Database;

/// Sort order for listing aliases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    /// Sort alphabetically by name (default)
    #[default]
    Alpha,
    /// Sort by usage count (most used first)
    Usage,
    /// Sort by last used time (most recent first)
    Recent,
}

impl From<&str> for SortOrder {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "usage" => SortOrder::Usage,
            "recent" => SortOrder::Recent,
            _ => SortOrder::Alpha,
        }
    }
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Alpha => write!(f, "alpha"),
            SortOrder::Usage => write!(f, "usage"),
            SortOrder::Recent => write!(f, "recent"),
        }
    }
}

/// List all aliases with optional sorting and filtering
pub fn list_with_options(
    db: &Database,
    config: &Config,
    sort_order: Option<&str>,
    filter_tag: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut aliases: Vec<_> = db.all().cloned().collect();

    // Filter by tag if specified
    if let Some(tag) = filter_tag {
        let tag_lower = tag.to_lowercase();
        aliases.retain(|a| a.tags.iter().any(|t| t.to_lowercase() == tag_lower));
    }

    if aliases.is_empty() {
        if filter_tag.is_some() {
            eprintln!("No aliases with tag '{}'", filter_tag.unwrap());
        } else {
            eprintln!("No aliases registered");
        }
        return Ok(());
    }

    // Determine sort order from argument or config default
    let order = sort_order
        .map(SortOrder::from)
        .unwrap_or_else(|| SortOrder::from(config.user.general.default_sort.as_str()));

    // Sort entries
    match order {
        SortOrder::Usage => aliases.sort_by(|a, b| b.use_count.cmp(&a.use_count)),
        SortOrder::Recent => aliases.sort_by(|a, b| b.last_used.cmp(&a.last_used)),
        SortOrder::Alpha => aliases.sort_by(|a, b| a.name.cmp(&b.name)),
    }

    // Print entries with formatted output
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for alias in &aliases {
        let tags_str = if config.user.display.show_tags {
            if alias.tags.is_empty() {
                String::new()
            } else {
                format!("    [{}]", alias.tags.join(", "))
            }
        } else {
            String::new()
        };

        if config.user.display.show_stats {
            writeln!(
                handle,
                "{:<12}    {}    [{} uses]{}",
                alias.name, alias.path, alias.use_count, tags_str
            )?;
        } else {
            writeln!(handle, "{:<12}    {}{}", alias.name, alias.path, tags_str)?;
        }
    }

    Ok(())
}

/// List all aliases with default options (uses config for display settings)
pub fn list(db: &Database, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    list_with_options(db, config, None, None)
}

/// List only alias names (one per line, for shell completion)
pub fn list_names(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let mut names: Vec<_> = db.names().collect();
    names.sort();

    for name in names {
        println!("{}", name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::Alias;
    use tempfile::tempdir;

    fn create_test_db_and_config() -> (Database, Config, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("aliases");
        let db = Database::load_from_path(&db_path).unwrap();
        let config = Config::load().unwrap();
        (db, config, dir)
    }

    #[test]
    fn test_sort_order_from_str() {
        assert_eq!(SortOrder::from("alpha"), SortOrder::Alpha);
        assert_eq!(SortOrder::from("ALPHA"), SortOrder::Alpha);
        assert_eq!(SortOrder::from("usage"), SortOrder::Usage);
        assert_eq!(SortOrder::from("USAGE"), SortOrder::Usage);
        assert_eq!(SortOrder::from("recent"), SortOrder::Recent);
        assert_eq!(SortOrder::from("RECENT"), SortOrder::Recent);
        assert_eq!(SortOrder::from("invalid"), SortOrder::Alpha); // default
    }

    #[test]
    fn test_sort_order_display() {
        assert_eq!(format!("{}", SortOrder::Alpha), "alpha");
        assert_eq!(format!("{}", SortOrder::Usage), "usage");
        assert_eq!(format!("{}", SortOrder::Recent), "recent");
    }

    #[test]
    fn test_list_empty() {
        let (db, config, _dir) = create_test_db_and_config();
        let result = list(&db, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list() {
        let (mut db, config, _dir) = create_test_db_and_config();
        db.insert(Alias::new("test", "/tmp").unwrap());

        let result = list(&db, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_names() {
        let (mut db, _config, _dir) = create_test_db_and_config();
        db.insert(Alias::new("alpha", "/tmp/a").unwrap());
        db.insert(Alias::new("beta", "/tmp/b").unwrap());

        let result = list_names(&db);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_with_sort_usage() {
        let (mut db, config, _dir) = create_test_db_and_config();

        let mut alias1 = Alias::new("low", "/tmp/low").unwrap();
        alias1.use_count = 1;
        db.insert(alias1);

        let mut alias2 = Alias::new("high", "/tmp/high").unwrap();
        alias2.use_count = 100;
        db.insert(alias2);

        // Should not error - output tested via integration tests
        let result = list_with_options(&db, &config, Some("usage"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_tag() {
        let (mut db, config, _dir) = create_test_db_and_config();

        let mut alias1 = Alias::new("work1", "/tmp/work1").unwrap();
        alias1.add_tag("work");
        db.insert(alias1);

        let mut alias2 = Alias::new("personal1", "/tmp/personal1").unwrap();
        alias2.add_tag("personal");
        db.insert(alias2);

        let mut alias3 = Alias::new("work2", "/tmp/work2").unwrap();
        alias3.add_tag("work");
        db.insert(alias3);

        // Filter by "work" tag
        let result = list_with_options(&db, &config, None, Some("work"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_filter_by_nonexistent_tag() {
        let (mut db, config, _dir) = create_test_db_and_config();
        db.insert(Alias::new("test", "/tmp").unwrap());

        // Filtering by non-existent tag should still succeed (just print message)
        let result = list_with_options(&db, &config, None, Some("nonexistent"));
        assert!(result.is_ok());
    }
}
