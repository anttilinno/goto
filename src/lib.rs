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
