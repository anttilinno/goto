//! Alias type and validation

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use thiserror::Error;

static VALID_ALIAS_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_.-]*$").unwrap());

static VALID_TAG_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9_-]*$").unwrap());

/// Errors that can occur during alias operations
#[derive(Error, Debug)]
pub enum AliasError {
    #[error("invalid alias '{alias}': {reason}")]
    InvalidAlias { alias: String, reason: String },

    #[error("alias '{0}' not found")]
    NotFound(String),

    #[error("alias '{0}' already exists")]
    AlreadyExists(String),

    #[error("directory does not exist: {0}")]
    DirectoryNotFound(String),

    #[error("invalid tag '{tag}': {reason}")]
    InvalidTag { tag: String, reason: String },
}

/// Validate that an alias name is acceptable
pub fn validate_alias(name: &str) -> Result<(), AliasError> {
    if name.is_empty() {
        return Err(AliasError::InvalidAlias {
            alias: name.to_string(),
            reason: "alias cannot be empty".to_string(),
        });
    }

    if !VALID_ALIAS_PATTERN.is_match(name) {
        return Err(AliasError::InvalidAlias {
            alias: name.to_string(),
            reason: "must start with letter/digit and contain only letters, digits, hyphens, underscores, dots".to_string(),
        });
    }

    Ok(())
}

/// Validate that a tag name is acceptable
pub fn validate_tag(tag: &str) -> Result<(), AliasError> {
    if tag.is_empty() {
        return Err(AliasError::InvalidTag {
            tag: tag.to_string(),
            reason: "tag cannot be empty".to_string(),
        });
    }

    if !VALID_TAG_PATTERN.is_match(tag) {
        return Err(AliasError::InvalidTag {
            tag: tag.to_string(),
            reason: "must start with letter/digit and contain only letters, digits, hyphens, underscores".to_string(),
        });
    }

    Ok(())
}

/// Represents a directory alias with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    /// The alias name
    pub name: String,
    /// The absolute path this alias points to
    pub path: String,
    /// Tags associated with this alias
    #[serde(default)]
    pub tags: Vec<String>,
    /// Number of times this alias has been used
    #[serde(default)]
    pub use_count: u64,
    /// Timestamp of last use
    #[serde(default)]
    pub last_used: Option<DateTime<Utc>>,
    /// Timestamp when the alias was created
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

impl Alias {
    /// Create a new alias with the given name and path
    pub fn new(name: &str, path: &str) -> Result<Self, AliasError> {
        validate_alias(name)?;
        Self::validate_path(path)?;

        Ok(Self {
            name: name.to_string(),
            path: path.to_string(),
            tags: Vec::new(),
            use_count: 0,
            last_used: None,
            created_at: Utc::now(),
        })
    }

    /// Validate that a path is acceptable
    pub fn validate_path(path: &str) -> Result<(), AliasError> {
        if path.is_empty() {
            return Err(AliasError::InvalidAlias {
                alias: String::new(),
                reason: "path cannot be empty".to_string(),
            });
        }

        Ok(())
    }

    /// Record a use of this alias
    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.last_used = Some(Utc::now());
    }

    /// Add a tag to this alias
    pub fn add_tag(&mut self, tag: &str) {
        let tag = tag.to_string();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.tags.sort();
        }
    }

    /// Remove a tag from this alias
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if this alias has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_alias() {
        let alias = Alias::new("projects", "/home/user/projects").unwrap();
        assert_eq!(alias.name, "projects");
        assert_eq!(alias.path, "/home/user/projects");
        assert!(alias.tags.is_empty());
        assert_eq!(alias.use_count, 0);
        assert!(alias.last_used.is_none());
    }

    #[test]
    fn test_invalid_name_empty() {
        let result = Alias::new("", "/home/user");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_name_starts_with_dash() {
        let result = Alias::new("-invalid", "/home/user");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_name_special_chars() {
        let result = Alias::new("hello world", "/home/user");
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_name_with_dash_underscore_dot() {
        let alias = Alias::new("my-project_v1.0", "/home/user/projects").unwrap();
        assert_eq!(alias.name, "my-project_v1.0");
    }

    #[test]
    fn test_record_use() {
        let mut alias = Alias::new("test", "/tmp").unwrap();
        assert_eq!(alias.use_count, 0);
        assert!(alias.last_used.is_none());

        alias.record_use();
        assert_eq!(alias.use_count, 1);
        assert!(alias.last_used.is_some());
    }

    #[test]
    fn test_tags() {
        let mut alias = Alias::new("test", "/tmp").unwrap();

        alias.add_tag("work");
        assert!(alias.has_tag("work"));
        assert!(!alias.has_tag("personal"));

        alias.add_tag("important");
        assert_eq!(alias.tags, vec!["important", "work"]); // sorted

        // Adding duplicate tag should not add it again
        alias.add_tag("work");
        assert_eq!(alias.tags.len(), 2);

        assert!(alias.remove_tag("work"));
        assert!(!alias.has_tag("work"));

        assert!(!alias.remove_tag("nonexistent"));
    }

    // Tests for validate_alias function
    #[test]
    fn test_validate_alias_empty() {
        let result = validate_alias("");
        assert!(matches!(result, Err(AliasError::InvalidAlias { .. })));
    }

    #[test]
    fn test_validate_alias_starts_with_dash() {
        let result = validate_alias("-invalid");
        assert!(matches!(result, Err(AliasError::InvalidAlias { .. })));
    }

    #[test]
    fn test_validate_alias_starts_with_underscore() {
        let result = validate_alias("_invalid");
        assert!(matches!(result, Err(AliasError::InvalidAlias { .. })));
    }

    #[test]
    fn test_validate_alias_valid_alphanumeric() {
        assert!(validate_alias("projects").is_ok());
        assert!(validate_alias("Projects123").is_ok());
        assert!(validate_alias("123projects").is_ok());
    }

    #[test]
    fn test_validate_alias_valid_with_special_chars() {
        assert!(validate_alias("my-project").is_ok());
        assert!(validate_alias("my_project").is_ok());
        assert!(validate_alias("my.project").is_ok());
        assert!(validate_alias("my-project_v1.0").is_ok());
    }

    #[test]
    fn test_validate_alias_invalid_special_chars() {
        assert!(validate_alias("hello world").is_err());
        assert!(validate_alias("hello@world").is_err());
        assert!(validate_alias("hello/world").is_err());
        assert!(validate_alias("hello:world").is_err());
    }

    // Tests for validate_tag function
    #[test]
    fn test_validate_tag_empty() {
        let result = validate_tag("");
        assert!(matches!(result, Err(AliasError::InvalidTag { .. })));
    }

    #[test]
    fn test_validate_tag_starts_with_dash() {
        let result = validate_tag("-invalid");
        assert!(matches!(result, Err(AliasError::InvalidTag { .. })));
    }

    #[test]
    fn test_validate_tag_starts_with_underscore() {
        let result = validate_tag("_invalid");
        assert!(matches!(result, Err(AliasError::InvalidTag { .. })));
    }

    #[test]
    fn test_validate_tag_valid() {
        assert!(validate_tag("work").is_ok());
        assert!(validate_tag("Work123").is_ok());
        assert!(validate_tag("my-tag").is_ok());
        assert!(validate_tag("my_tag").is_ok());
    }

    #[test]
    fn test_validate_tag_invalid_dot() {
        // Tags don't allow dots (unlike aliases)
        let result = validate_tag("my.tag");
        assert!(matches!(result, Err(AliasError::InvalidTag { .. })));
    }

    #[test]
    fn test_validate_tag_invalid_special_chars() {
        assert!(validate_tag("hello world").is_err());
        assert!(validate_tag("hello@world").is_err());
        assert!(validate_tag("hello/world").is_err());
    }

    // Tests for error messages
    #[test]
    fn test_error_messages() {
        let err = AliasError::NotFound("test".to_string());
        assert_eq!(format!("{}", err), "alias 'test' not found");

        let err = AliasError::AlreadyExists("test".to_string());
        assert_eq!(format!("{}", err), "alias 'test' already exists");

        let err = AliasError::DirectoryNotFound("/nonexistent".to_string());
        assert_eq!(format!("{}", err), "directory does not exist: /nonexistent");

        let err = AliasError::InvalidAlias {
            alias: "bad".to_string(),
            reason: "test reason".to_string(),
        };
        assert_eq!(format!("{}", err), "invalid alias 'bad': test reason");

        let err = AliasError::InvalidTag {
            tag: "bad".to_string(),
            reason: "test reason".to_string(),
        };
        assert_eq!(format!("{}", err), "invalid tag 'bad': test reason");
    }
}
