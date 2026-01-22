//! Statistics commands: stats, recent, clear_recent

use chrono::{DateTime, Utc};

use crate::config::Config;
use crate::database::Database;
use crate::table::{TableStyle, create_table};

/// Recent entry for display
pub struct RecentEntry {
    pub alias: String,
    pub path: String,
    pub last_used: DateTime<Utc>,
}

/// Show usage statistics
pub fn stats(db: &Database, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if db.is_empty() {
        println!("No aliases registered");
        return Ok(());
    }

    // Sort by use count descending
    let mut entries: Vec<_> = db.all().collect();
    entries.sort_by(|a, b| b.use_count.cmp(&a.use_count));

    // Calculate total navigations
    let total_navigations: u64 = entries.iter().map(|e| e.use_count).sum();

    println!("Usage Statistics");
    println!();

    // Filter to only used entries and take top 10
    let used_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.use_count > 0)
        .take(10)
        .collect();

    if used_entries.is_empty() {
        println!("(no aliases have been used yet)");
    } else {
        let style = TableStyle::from(config.user.display.table_style.as_str());
        let mut table = create_table(style);
        table.set_header(vec!["#", "Name", "Uses", "Last Used"]);

        for (i, entry) in used_entries.iter().enumerate() {
            let last_used_str = format_time_ago(entry.last_used);
            table.add_row(vec![
                (i + 1).to_string(),
                entry.name.clone(),
                entry.use_count.to_string(),
                last_used_str,
            ]);
        }

        println!("{table}");
    }

    println!();
    println!("Total aliases: {}", entries.len());
    println!("Total navigations: {}", total_navigations);

    Ok(())
}

/// Get recently visited aliases sorted by last_used descending
pub fn recent(db: &Database, limit: Option<usize>) -> Result<Vec<RecentEntry>, Box<dyn std::error::Error>> {
    // Filter to only entries that have been used
    let mut used_entries: Vec<_> = db.all().filter(|e| e.last_used.is_some()).collect();

    if used_entries.is_empty() {
        return Ok(Vec::new());
    }

    // Sort by last_used descending
    used_entries.sort_by(|a, b| b.last_used.cmp(&a.last_used));

    // Limit results
    if let Some(limit) = limit {
        used_entries.truncate(limit);
    }

    Ok(used_entries
        .into_iter()
        .map(|e| RecentEntry {
            alias: e.name.clone(),
            path: e.path.clone(),
            last_used: e.last_used.unwrap(),
        })
        .collect())
}

/// Display recently visited aliases
pub fn show_recent(db: &Database, config: &Config, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    let limit = if limit == 0 { 10 } else { limit };
    let entries = recent(db, Some(limit))?;

    if entries.is_empty() {
        println!("No recently visited directories");
        return Ok(());
    }

    let style = TableStyle::from(config.user.display.table_style.as_str());
    let mut table = create_table(style);
    table.set_header(vec!["#", "Name", "Path", "Last Visited"]);

    for (i, entry) in entries.iter().enumerate() {
        let time_ago = format_time_ago(Some(entry.last_used));
        table.add_row(vec![
            (i + 1).to_string(),
            entry.alias.clone(),
            entry.path.clone(),
            time_ago,
        ]);
    }

    println!("{table}");

    Ok(())
}

/// Navigate to the Nth most recent alias
pub fn navigate_to_recent(db: &mut Database, index: usize) -> Result<(), Box<dyn std::error::Error>> {
    let entries = recent(db, None)?;

    if entries.is_empty() {
        return Err("no recently visited directories".into());
    }

    if index < 1 || index > entries.len() {
        return Err(format!(
            "invalid recent index: {} (valid: 1-{})",
            index,
            entries.len()
        )
        .into());
    }

    // Navigate to the alias
    crate::commands::navigate::navigate(db, &entries[index - 1].alias)
}

/// Clear recent history (reset last_used for all aliases)
pub fn clear_recent(db: &mut Database) -> Result<(), Box<dyn std::error::Error>> {
    db.clear_recent_history()?;
    db.save()?;
    println!("Cleared recent history");
    Ok(())
}

/// Format a timestamp as a human-readable "time ago" string
fn format_time_ago(t: Option<DateTime<Utc>>) -> String {
    let t = match t {
        Some(t) => t,
        None => return "never".to_string(),
    };

    let duration = Utc::now().signed_duration_since(t);

    if duration.num_seconds() < 60 {
        return "just now".to_string();
    }

    let minutes = duration.num_minutes();
    if minutes < 60 {
        return if minutes == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", minutes)
        };
    }

    let hours = duration.num_hours();
    if hours < 24 {
        return if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", hours)
        };
    }

    let days = duration.num_days();
    if days < 7 {
        return if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        };
    }

    let weeks = days / 7;
    if weeks < 4 {
        return if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format!("{} weeks ago", weeks)
        };
    }

    let months = days / 30;
    if months == 1 {
        "1 month ago".to_string()
    } else {
        format!("{} months ago", months)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::Alias;
    use crate::config::Config;
    use chrono::Duration;
    use tempfile::NamedTempFile;

    fn create_test_db() -> (Database, NamedTempFile) {
        let file = NamedTempFile::new().unwrap();
        let mut db = Database::load_from_path(file.path()).unwrap();

        let mut a1 = Alias::new("often", "/tmp/often").unwrap();
        for _ in 0..10 {
            a1.record_use();
        }
        db.insert(a1);

        let mut a2 = Alias::new("sometimes", "/tmp/sometimes").unwrap();
        for _ in 0..3 {
            a2.record_use();
        }
        db.insert(a2);

        db.insert(Alias::new("never", "/tmp/never").unwrap());

        (db, file)
    }

    #[test]
    fn test_stats() {
        let (db, _file) = create_test_db();
        let config = Config::load().unwrap();
        let result = stats(&db, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stats_empty() {
        let file = NamedTempFile::new().unwrap();
        let db = Database::load_from_path(file.path()).unwrap();
        let config = Config::load().unwrap();
        let result = stats(&db, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recent_returns_entries() {
        let (db, _file) = create_test_db();
        let entries = recent(&db, Some(5)).unwrap();
        // "often" and "sometimes" have been used, "never" has not
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_recent_sorted_by_last_used() {
        let file = NamedTempFile::new().unwrap();
        let mut db = Database::load_from_path(file.path()).unwrap();

        // Create aliases and use them in order
        let mut a1 = Alias::new("first", "/tmp/first").unwrap();
        a1.record_use();
        db.insert(a1);

        // Small delay simulation - just use different last_used
        let mut a2 = Alias::new("second", "/tmp/second").unwrap();
        a2.record_use();
        db.insert(a2);

        let entries = recent(&db, None).unwrap();
        // Most recent should be first
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].alias, "second");
        assert_eq!(entries[1].alias, "first");
    }

    #[test]
    fn test_recent_with_limit() {
        let (db, _file) = create_test_db();
        let entries = recent(&db, Some(1)).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_recent_empty() {
        let file = NamedTempFile::new().unwrap();
        let db = Database::load_from_path(file.path()).unwrap();
        let entries = recent(&db, None).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_show_recent() {
        let (db, _file) = create_test_db();
        let config = Config::load().unwrap();
        let result = show_recent(&db, &config, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_recent_empty() {
        let file = NamedTempFile::new().unwrap();
        let db = Database::load_from_path(file.path()).unwrap();
        let config = Config::load().unwrap();
        let result = show_recent(&db, &config, 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_navigate_to_recent_invalid_index() {
        let (mut db, _file) = create_test_db();

        // Index 0 is invalid
        let result = navigate_to_recent(&mut db, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid recent index"));

        // Index too high
        let result = navigate_to_recent(&mut db, 100);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid recent index"));
    }

    #[test]
    fn test_navigate_to_recent_empty() {
        let file = NamedTempFile::new().unwrap();
        let mut db = Database::load_from_path(file.path()).unwrap();

        let result = navigate_to_recent(&mut db, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no recently visited"));
    }

    #[test]
    fn test_clear_recent() {
        let (mut db, _file) = create_test_db();

        // Verify we have recent entries
        let entries = recent(&db, None).unwrap();
        assert!(!entries.is_empty());

        // Clear history
        let result = clear_recent(&mut db);
        assert!(result.is_ok());

        // Verify no recent entries
        let entries = recent(&db, None).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_format_time_ago_none() {
        assert_eq!(format_time_ago(None), "never");
    }

    #[test]
    fn test_format_time_ago_just_now() {
        let now = Utc::now();
        assert_eq!(format_time_ago(Some(now)), "just now");
    }

    #[test]
    fn test_format_time_ago_minutes() {
        let time = Utc::now() - Duration::minutes(1);
        assert_eq!(format_time_ago(Some(time)), "1 minute ago");

        let time = Utc::now() - Duration::minutes(30);
        assert_eq!(format_time_ago(Some(time)), "30 minutes ago");
    }

    #[test]
    fn test_format_time_ago_hours() {
        let time = Utc::now() - Duration::hours(1);
        assert_eq!(format_time_ago(Some(time)), "1 hour ago");

        let time = Utc::now() - Duration::hours(5);
        assert_eq!(format_time_ago(Some(time)), "5 hours ago");
    }

    #[test]
    fn test_format_time_ago_days() {
        let time = Utc::now() - Duration::days(1);
        assert_eq!(format_time_ago(Some(time)), "1 day ago");

        let time = Utc::now() - Duration::days(3);
        assert_eq!(format_time_ago(Some(time)), "3 days ago");
    }

    #[test]
    fn test_format_time_ago_weeks() {
        let time = Utc::now() - Duration::weeks(1);
        assert_eq!(format_time_ago(Some(time)), "1 week ago");

        let time = Utc::now() - Duration::weeks(2);
        assert_eq!(format_time_ago(Some(time)), "2 weeks ago");
    }

    #[test]
    fn test_format_time_ago_months() {
        let time = Utc::now() - Duration::days(30);
        assert_eq!(format_time_ago(Some(time)), "1 month ago");

        let time = Utc::now() - Duration::days(90);
        assert_eq!(format_time_ago(Some(time)), "3 months ago");
    }
}
