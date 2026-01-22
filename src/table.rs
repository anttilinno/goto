//! Table formatting utilities for consistent display output
//!
//! This module provides a thin abstraction over comfy-table that ensures
//! consistent table styling across all display commands.

use comfy_table::{presets, modifiers, ContentArrangement, Table};

/// Table display style options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TableStyle {
    /// Unicode box-drawing characters with rounded corners (default)
    #[default]
    Unicode,
    /// ASCII-only characters for maximum compatibility
    Ascii,
    /// Minimal style with no borders
    Minimal,
}

impl From<&str> for TableStyle {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "unicode" => TableStyle::Unicode,
            "ascii" => TableStyle::Ascii,
            "minimal" | "none" => TableStyle::Minimal,
            _ => TableStyle::Unicode, // Unknown values fall back to unicode
        }
    }
}

/// Create a new table with the specified style
///
/// Returns a configured `comfy_table::Table` with:
/// - The appropriate style preset applied
/// - Dynamic content arrangement enabled
pub fn create_table(style: TableStyle) -> Table {
    let mut table = Table::new();

    match style {
        TableStyle::Unicode => {
            table
                .load_preset(presets::UTF8_FULL)
                .apply_modifier(modifiers::UTF8_ROUND_CORNERS);
        }
        TableStyle::Ascii => {
            table.load_preset(presets::ASCII_FULL);
        }
        TableStyle::Minimal => {
            table.load_preset(presets::NOTHING);
        }
    }

    table.set_content_arrangement(ContentArrangement::Dynamic);
    table
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_style_from_str() {
        // Lowercase
        assert_eq!(TableStyle::from("unicode"), TableStyle::Unicode);
        assert_eq!(TableStyle::from("ascii"), TableStyle::Ascii);
        assert_eq!(TableStyle::from("minimal"), TableStyle::Minimal);
        assert_eq!(TableStyle::from("none"), TableStyle::Minimal);

        // Uppercase
        assert_eq!(TableStyle::from("UNICODE"), TableStyle::Unicode);
        assert_eq!(TableStyle::from("ASCII"), TableStyle::Ascii);
        assert_eq!(TableStyle::from("MINIMAL"), TableStyle::Minimal);

        // Mixed case
        assert_eq!(TableStyle::from("Unicode"), TableStyle::Unicode);
        assert_eq!(TableStyle::from("Ascii"), TableStyle::Ascii);

        // Invalid values fall back to Unicode
        assert_eq!(TableStyle::from("invalid"), TableStyle::Unicode);
        assert_eq!(TableStyle::from(""), TableStyle::Unicode);
        assert_eq!(TableStyle::from("fancy"), TableStyle::Unicode);
    }

    #[test]
    fn test_table_style_default() {
        assert_eq!(TableStyle::default(), TableStyle::Unicode);
    }

    #[test]
    fn test_create_table_returns_table() {
        // Smoke test: verify create_table returns a table for each style
        let _unicode = create_table(TableStyle::Unicode);
        let _ascii = create_table(TableStyle::Ascii);
        let _minimal = create_table(TableStyle::Minimal);
    }

    #[test]
    fn test_create_table_with_data() {
        let mut table = create_table(TableStyle::Unicode);
        table.set_header(vec!["Name", "Path"]);
        table.add_row(vec!["projects", "/home/user/projects"]);
        table.add_row(vec!["downloads", "/home/user/downloads"]);

        let output = table.to_string();
        assert!(!output.is_empty());
        assert!(output.contains("projects"));
        assert!(output.contains("downloads"));
        assert!(output.contains("Name"));
        assert!(output.contains("Path"));
    }
}
