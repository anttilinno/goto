//! goto - Navigate to aliased directories with autocomplete support
//!
//! This library provides functionality for managing directory aliases,
//! enabling quick navigation between frequently used directories.

use std::io::{self, IsTerminal, Write};

pub mod alias;
pub mod cli;
pub mod commands;
pub mod config;
pub mod database;
pub mod fuzzy;
pub mod stack;
pub mod table;

pub use alias::Alias;
pub use cli::{parse_args, Args, Command};
pub use config::Config;
pub use database::Database;
pub use stack::Stack;
pub use table::{TableStyle, create_table};

/// Prompt user for y/n confirmation.
///
/// Returns the default value if stdin is not a terminal (for piped/non-interactive use).
/// On a terminal, displays the message with (Y/n) or (y/N) suffix based on default,
/// then parses user input: empty returns default, y/yes returns true, n/no returns false.
///
/// # Arguments
/// * `message` - The prompt message to display
/// * `default` - The default value returned on empty input or non-terminal
///
/// # Returns
/// * `Ok(true)` - User confirmed (y/yes) or default was true with empty input
/// * `Ok(false)` - User declined (n/no) or default was false with empty input
/// * `Err` - I/O error occurred
pub fn confirm(message: &str, default: bool) -> io::Result<bool> {
    if !io::stdin().is_terminal() {
        return Ok(default);
    }

    let suffix = if default { "(Y/n)" } else { "(y/N)" };
    print!("{} {} ", message, suffix);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();

    Ok(match input.as_str() {
        "" => default,
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    })
}

/// Prompt user to select from numbered options.
///
/// Returns the selected index (0-based) on valid input, None on cancel.
/// Returns None immediately if stdin is not a terminal (non-interactive mode).
///
/// # Arguments
/// * `options` - List of option labels to display
/// * `similarity_scores` - Optional scores to display as percentages (0.0-1.0)
///
/// # Returns
/// * `Ok(Some(index))` - User selected option at index
/// * `Ok(None)` - User cancelled (Enter or invalid input) or non-interactive
/// * `Err` - I/O error occurred
pub fn prompt_selection(
    options: &[&str],
    similarity_scores: Option<&[f64]>,
) -> io::Result<Option<usize>> {
    // Non-interactive mode: return None immediately
    if !io::stdin().is_terminal() {
        return Ok(None);
    }

    // Display options with numbers
    for (i, option) in options.iter().enumerate() {
        if let Some(scores) = similarity_scores {
            if let Some(score) = scores.get(i) {
                let percentage = (score * 100.0).round() as u32;
                eprintln!("  [{}] {} ({}% match)", i + 1, option, percentage);
            } else {
                eprintln!("  [{}] {}", i + 1, option);
            }
        } else {
            eprintln!("  [{}] {}", i + 1, option);
        }
    }

    eprint!("Select [1-{}] or Enter to cancel: ", options.len());
    io::stderr().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim();

    // Empty input = cancel
    if input.is_empty() {
        return Ok(None);
    }

    // Try to parse as number
    match input.parse::<usize>() {
        Ok(n) if n >= 1 && n <= options.len() => Ok(Some(n - 1)),
        _ => Ok(None), // Invalid input = cancel
    }
}
