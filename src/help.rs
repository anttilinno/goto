//! Help system infrastructure for tiered help output.
//!
//! Provides types and formatting functions for the three help modes:
//! - Brief help (`--help`): Essential commands only
//! - Full help (`--help-all`): All commands grouped by category
//! - Per-command help (`--help <command>`): Detailed help for one command

/// Metadata for a single command's help.
pub struct CommandHelp {
    /// Short flag (e.g., "-r")
    pub short_flag: Option<&'static str>,
    /// Long flag (e.g., "--register")
    pub long_flag: &'static str,
    /// One-line description
    pub description: &'static str,
    /// Detailed explanation
    pub detailed: &'static str,
    /// Usage pattern (e.g., "goto -r \<alias\> \<path\>")
    pub usage: &'static str,
    /// Sub-options
    pub options: &'static [CommandOption],
    /// Runnable examples
    pub examples: &'static [Example],
    /// Related commands
    pub see_also: &'static [&'static str],
    /// For grouping
    pub category: CommandCategory,
    /// Show in brief help?
    pub essential: bool,
}

/// A sub-option for a command.
pub struct CommandOption {
    /// Flag (e.g., "--force")
    pub flag: &'static str,
    /// Description of what it does
    pub description: &'static str,
}

/// An example usage of a command.
pub struct Example {
    /// Description of what this example demonstrates
    pub description: &'static str,
    /// The actual command to run
    pub command: &'static str,
}

/// Category for grouping commands in help output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    /// Navigate, register, list, tags
    Essential,
    /// Cleanup, rename, import/export
    Management,
    /// Stats, config, version
    Info,
    /// Install, update, prune-snooze
    Advanced,
}

impl CommandCategory {
    /// Human-readable title for this category.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Essential => "Essential Commands",
            Self::Management => "Alias Management",
            Self::Info => "Information",
            Self::Advanced => "Advanced",
        }
    }

    /// All categories in display order.
    pub fn all() -> &'static [CommandCategory] {
        &[
            CommandCategory::Essential,
            CommandCategory::Management,
            CommandCategory::Info,
            CommandCategory::Advanced,
        ]
    }
}

/// Registry of all commands (populated in Plan 02).
pub static COMMANDS: &[CommandHelp] = &[
    // Will be populated in Plan 02
];

/// Flag column width for alignment.
const FLAG_WIDTH: usize = 28;

/// Print brief help (essential commands only).
///
/// Shows header, essential commands, and footer with hints for more help.
pub fn print_brief_help() {
    println!("goto - Navigate to aliased directories");
    println!();
    println!("Usage: goto <alias> | goto [OPTIONS]");
    println!();

    // Essential commands
    let essential: Vec<_> = COMMANDS.iter().filter(|c| c.essential).collect();

    if !essential.is_empty() {
        println!("Common Commands:");
        for cmd in essential {
            print_command_line(cmd);
        }
        println!();
    }

    // Footer with hints
    println!("Run 'goto --help-all' for all commands");
    println!("Run 'goto --help <command>' for detailed help on a command");
}

/// Print full help (all commands grouped by category).
///
/// Shows header and all commands organized by their category.
pub fn print_full_help() {
    println!("goto - Navigate to aliased directories");
    println!();
    println!("Usage: goto <alias> | goto [OPTIONS]");

    // Group commands by category
    for category in CommandCategory::all() {
        let commands: Vec<_> = COMMANDS
            .iter()
            .filter(|c| c.category == *category)
            .collect();

        if !commands.is_empty() {
            println!();
            println!("{}:", category.title());
            for cmd in commands {
                print_command_line(cmd);
            }
        }
    }

    if !COMMANDS.is_empty() {
        println!();
        println!("Run 'goto --help <command>' for detailed help on a command");
    }
}

/// Print detailed help for a specific command.
///
/// # Arguments
/// * `command` - Command name to look up (short flag, long flag, or name)
///
/// # Returns
/// * `Ok(())` - Help was printed
/// * `Err(String)` - Command not found, with suggestion
pub fn print_command_help(command: &str) -> Result<(), String> {
    let cmd = find_command(command).ok_or_else(|| {
        format!("Unknown command: {}\nRun 'goto --help-all' to see all commands", command)
    })?;

    // Header with flags
    if let Some(short) = cmd.short_flag {
        println!("{}, {}", short, cmd.long_flag);
    } else {
        println!("{}", cmd.long_flag);
    }
    println!();

    // Description
    println!("{}", cmd.description);
    println!();

    // Detailed explanation
    if !cmd.detailed.is_empty() {
        println!("{}", cmd.detailed);
        println!();
    }

    // Usage
    println!("Usage:");
    println!("  {}", cmd.usage);
    println!();

    // Options
    if !cmd.options.is_empty() {
        println!("Options:");
        for opt in cmd.options {
            println!("  {:24} {}", opt.flag, opt.description);
        }
        println!();
    }

    // Examples
    if !cmd.examples.is_empty() {
        println!("Examples:");
        for ex in cmd.examples {
            println!("  {} - {}", ex.command, ex.description);
        }
        println!();
    }

    // See also
    if !cmd.see_also.is_empty() {
        println!("See also: {}", cmd.see_also.join(", "));
    }

    Ok(())
}

/// Print a single command line for help listing.
///
/// Formats flags and description with consistent alignment.
fn print_command_line(cmd: &CommandHelp) {
    let flags = if let Some(short) = cmd.short_flag {
        format!("  {}, {}", short, cmd.long_flag)
    } else {
        format!("  {}", cmd.long_flag)
    };

    // Pad to FLAG_WIDTH, then print description
    if flags.len() >= FLAG_WIDTH {
        println!("{}", flags);
        println!("{:FLAG_WIDTH$}{}", "", cmd.description);
    } else {
        println!("{:FLAG_WIDTH$}{}", flags, cmd.description);
    }
}

/// Find a command by short flag, long flag, or name.
///
/// Matches against:
/// - Short flag (e.g., "-r")
/// - Long flag (e.g., "--register")
/// - Long flag without dashes (e.g., "register")
fn find_command(name: &str) -> Option<&'static CommandHelp> {
    let normalized = name.trim_start_matches('-');

    COMMANDS.iter().find(|cmd| {
        // Match short flag
        if let Some(short) = cmd.short_flag {
            if short == name || short.trim_start_matches('-') == normalized {
                return true;
            }
        }
        // Match long flag
        cmd.long_flag == name || cmd.long_flag.trim_start_matches('-') == normalized
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_title() {
        assert_eq!(CommandCategory::Essential.title(), "Essential Commands");
        assert_eq!(CommandCategory::Management.title(), "Alias Management");
        assert_eq!(CommandCategory::Info.title(), "Information");
        assert_eq!(CommandCategory::Advanced.title(), "Advanced");
    }

    #[test]
    fn test_category_all_order() {
        let all = CommandCategory::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], CommandCategory::Essential);
        assert_eq!(all[1], CommandCategory::Management);
        assert_eq!(all[2], CommandCategory::Info);
        assert_eq!(all[3], CommandCategory::Advanced);
    }

    #[test]
    fn test_find_command_empty_registry() {
        // With empty COMMANDS, find_command should return None
        assert!(find_command("register").is_none());
        assert!(find_command("-r").is_none());
        assert!(find_command("--register").is_none());
    }

    #[test]
    fn test_print_command_help_not_found() {
        let result = print_command_help("nonexistent");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown command"));
        assert!(err.contains("--help-all"));
    }
}
