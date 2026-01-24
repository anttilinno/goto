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

/// Registry of all commands.
pub static COMMANDS: &[CommandHelp] = &[
    // ==================== ESSENTIAL ====================
    CommandHelp {
        short_flag: None,
        long_flag: "<alias>",
        description: "Navigate to an aliased directory",
        detailed: "Changes to the directory associated with the given alias. \
                   If the alias doesn't exist, suggests similar aliases. \
                   This is the default action when no flags are provided.",
        usage: "goto <alias>",
        options: &[],
        examples: &[
            Example { description: "Navigate to project directory", command: "goto myproject" },
            Example { description: "Navigate to development folder", command: "goto dev" },
        ],
        see_also: &["--list", "--register", "--recent"],
        category: CommandCategory::Essential,
        essential: true,
    },
    CommandHelp {
        short_flag: Some("-l"),
        long_flag: "--list",
        description: "List all registered aliases",
        detailed: "Displays a table of all aliases with their paths, tags, and usage statistics. \
                   Results can be sorted and filtered using optional flags.",
        usage: "goto -l [--sort=<order>] [--filter=<tag>]",
        options: &[
            CommandOption { flag: "--sort=<order>", description: "Sort by: alpha, usage, recent" },
            CommandOption { flag: "--filter=<tag>", description: "Show only aliases with tag" },
        ],
        examples: &[
            Example { description: "List all aliases", command: "goto -l" },
            Example { description: "List sorted by usage", command: "goto -l --sort=usage" },
            Example { description: "Filter by tag", command: "goto -l --filter=work" },
        ],
        see_also: &["--register", "--tags", "--stats"],
        category: CommandCategory::Essential,
        essential: true,
    },
    CommandHelp {
        short_flag: Some("-r"),
        long_flag: "--register",
        description: "Register a new alias for a directory",
        detailed: "Creates a new alias pointing to a directory. The alias name must be \
                   alphanumeric (with dashes/underscores allowed). The directory must exist. \
                   Use --force to skip confirmation when creating new tags.",
        usage: "goto -r <alias> <directory> [-t <tags>] [--force]",
        options: &[
            CommandOption { flag: "-t, --tags=<tags>", description: "Comma-separated tags to assign" },
            CommandOption { flag: "-f, --force", description: "Skip confirmation for new tags" },
        ],
        examples: &[
            Example { description: "Register current directory", command: "goto -r myproject ." },
            Example { description: "Register with tags", command: "goto -r work ~/work -t job,important" },
        ],
        see_also: &["--unregister", "--list", "--tag"],
        category: CommandCategory::Essential,
        essential: true,
    },
    CommandHelp {
        short_flag: Some("-T"),
        long_flag: "--tags",
        description: "List all tags with usage counts",
        detailed: "Shows a table of all tags used across aliases, with the count of \
                   aliases using each tag. Useful for organizing and filtering aliases.",
        usage: "goto -T",
        options: &[],
        examples: &[
            Example { description: "Show all tags", command: "goto -T" },
        ],
        see_also: &["--tag", "--untag", "--list"],
        category: CommandCategory::Essential,
        essential: true,
    },
    // ==================== MANAGEMENT ====================
    CommandHelp {
        short_flag: Some("-c"),
        long_flag: "--cleanup",
        description: "Remove aliases pointing to non-existent directories",
        detailed: "Scans all aliases and removes those whose target directories no longer exist. \
                   Use --dry-run to preview what would be removed without making changes.",
        usage: "goto -c [--dry-run]",
        options: &[
            CommandOption { flag: "--dry-run", description: "Preview changes without removing" },
        ],
        examples: &[
            Example { description: "Clean up invalid aliases", command: "goto -c" },
            Example { description: "Preview cleanup", command: "goto -c --dry-run" },
        ],
        see_also: &["--list", "--unregister"],
        category: CommandCategory::Management,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-e"),
        long_flag: "--export",
        description: "Export aliases to TOML format",
        detailed: "Outputs all aliases in TOML format to stdout. \
                   Redirect to a file to create a backup or share aliases.",
        usage: "goto -e",
        options: &[],
        examples: &[
            Example { description: "Export to file", command: "goto -e > backup.toml" },
        ],
        see_also: &["--import"],
        category: CommandCategory::Management,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-i"),
        long_flag: "--import",
        description: "Import aliases from a TOML file",
        detailed: "Reads aliases from a TOML file and adds them to the database. \
                   The strategy option controls how conflicts are handled.",
        usage: "goto -i <file> [--strategy=<strategy>]",
        options: &[
            CommandOption { flag: "--strategy=skip", description: "Skip existing aliases (default)" },
            CommandOption { flag: "--strategy=overwrite", description: "Overwrite existing aliases" },
            CommandOption { flag: "--strategy=rename", description: "Rename conflicting aliases" },
        ],
        examples: &[
            Example { description: "Import from backup", command: "goto -i backup.toml" },
            Example { description: "Import and overwrite", command: "goto -i backup.toml --strategy=overwrite" },
        ],
        see_also: &["--export"],
        category: CommandCategory::Management,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-o"),
        long_flag: "--pop",
        description: "Pop and return to the previous directory",
        detailed: "Returns to the directory saved by the last --push command. \
                   The directory stack is persisted across sessions.",
        usage: "goto -o",
        options: &[],
        examples: &[
            Example { description: "Return to previous directory", command: "goto -o" },
        ],
        see_also: &["--push"],
        category: CommandCategory::Management,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-p"),
        long_flag: "--push",
        description: "Save current directory and navigate to alias",
        detailed: "Pushes the current working directory onto the stack, then navigates \
                   to the specified alias. Use --pop to return later.",
        usage: "goto -p <alias>",
        options: &[],
        examples: &[
            Example { description: "Save location and go to work", command: "goto -p work" },
        ],
        see_also: &["--pop"],
        category: CommandCategory::Management,
        essential: false,
    },
    CommandHelp {
        short_flag: None,
        long_flag: "--rename",
        description: "Rename an existing alias",
        detailed: "Changes the name of an alias while preserving its path, tags, and statistics. \
                   The new name must not already exist.",
        usage: "goto --rename <old-alias> <new-alias>",
        options: &[],
        examples: &[
            Example { description: "Rename project to proj", command: "goto --rename project proj" },
        ],
        see_also: &["--register", "--unregister"],
        category: CommandCategory::Management,
        essential: false,
    },
    CommandHelp {
        short_flag: None,
        long_flag: "--rename-tag",
        description: "Rename a tag across all aliases",
        detailed: "Changes a tag name on all aliases that use it. Useful for reorganizing \
                   your tag taxonomy. Use --dry-run to preview changes first.",
        usage: "goto --rename-tag <old-tag> <new-tag> [--dry-run] [--force]",
        options: &[
            CommandOption { flag: "--dry-run", description: "Preview changes without applying" },
            CommandOption { flag: "-f, --force", description: "Skip confirmation prompt" },
        ],
        examples: &[
            Example { description: "Rename tag", command: "goto --rename-tag javascript js" },
            Example { description: "Preview rename", command: "goto --rename-tag old new --dry-run" },
        ],
        see_also: &["--tag", "--untag", "--tags"],
        category: CommandCategory::Management,
        essential: false,
    },
    CommandHelp {
        short_flag: None,
        long_flag: "--tag",
        description: "Add a tag to an alias",
        detailed: "Adds a tag to an existing alias. If the tag doesn't exist yet, \
                   you'll be prompted to confirm. Use --force to skip confirmation.",
        usage: "goto --tag <alias> <tag> [--force]",
        options: &[
            CommandOption { flag: "-f, --force", description: "Skip confirmation for new tags" },
        ],
        examples: &[
            Example { description: "Add tag to alias", command: "goto --tag myproject work" },
            Example { description: "Add new tag without confirmation", command: "goto --tag myproject newtag -f" },
        ],
        see_also: &["--untag", "--tags", "--register"],
        category: CommandCategory::Management,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-u"),
        long_flag: "--unregister",
        description: "Remove an alias",
        detailed: "Deletes an alias from the database. The target directory is not affected.",
        usage: "goto -u <alias>",
        options: &[],
        examples: &[
            Example { description: "Remove an alias", command: "goto -u oldproject" },
        ],
        see_also: &["--register", "--cleanup"],
        category: CommandCategory::Management,
        essential: false,
    },
    CommandHelp {
        short_flag: None,
        long_flag: "--untag",
        description: "Remove a tag from an alias",
        detailed: "Removes a specific tag from an alias. The tag itself may still exist \
                   on other aliases.",
        usage: "goto --untag <alias> <tag>",
        options: &[],
        examples: &[
            Example { description: "Remove tag from alias", command: "goto --untag myproject oldtag" },
        ],
        see_also: &["--tag", "--tags"],
        category: CommandCategory::Management,
        essential: false,
    },
    // ==================== INFO ====================
    CommandHelp {
        short_flag: None,
        long_flag: "--config",
        description: "Show current configuration",
        detailed: "Displays the current configuration values and file location. \
                   Edit the config file to change settings.",
        usage: "goto --config",
        options: &[],
        examples: &[
            Example { description: "Show configuration", command: "goto --config" },
        ],
        see_also: &["--stats"],
        category: CommandCategory::Info,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-R"),
        long_flag: "--recent",
        description: "Show or navigate to recently visited aliases",
        detailed: "Without arguments, shows a numbered list of recently visited aliases. \
                   With a number, navigates to that entry in the list.",
        usage: "goto -R [<number>]",
        options: &[],
        examples: &[
            Example { description: "Show recent aliases", command: "goto -R" },
            Example { description: "Navigate to 3rd most recent", command: "goto -R 3" },
        ],
        see_also: &["--recent-clear", "--stats"],
        category: CommandCategory::Info,
        essential: false,
    },
    CommandHelp {
        short_flag: None,
        long_flag: "--recent-clear",
        description: "Clear recent navigation history",
        detailed: "Removes all entries from the recent navigation history.",
        usage: "goto --recent-clear",
        options: &[],
        examples: &[
            Example { description: "Clear recent history", command: "goto --recent-clear" },
        ],
        see_also: &["--recent"],
        category: CommandCategory::Info,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-s"),
        long_flag: "--stats",
        description: "Show usage statistics",
        detailed: "Displays statistics about your aliases including total count, \
                   most used aliases, and tag distribution.",
        usage: "goto -s",
        options: &[],
        examples: &[
            Example { description: "Show statistics", command: "goto -s" },
        ],
        see_also: &["--list", "--recent"],
        category: CommandCategory::Info,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-v"),
        long_flag: "--version",
        description: "Show version information",
        detailed: "Displays the current version of goto and checks for available updates.",
        usage: "goto -v",
        options: &[],
        examples: &[
            Example { description: "Show version", command: "goto -v" },
        ],
        see_also: &["--update", "--check-update"],
        category: CommandCategory::Info,
        essential: false,
    },
    // ==================== ADVANCED ====================
    CommandHelp {
        short_flag: None,
        long_flag: "--check-update",
        description: "Check for available updates",
        detailed: "Checks if a newer version of goto is available without installing it.",
        usage: "goto --check-update",
        options: &[],
        examples: &[
            Example { description: "Check for updates", command: "goto --check-update" },
        ],
        see_also: &["--update", "--version"],
        category: CommandCategory::Advanced,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-x"),
        long_flag: "--expand",
        description: "Expand alias to its full path",
        detailed: "Outputs the full directory path for an alias without navigating. \
                   Useful in scripts or for copying the path.",
        usage: "goto -x <alias>",
        options: &[],
        examples: &[
            Example { description: "Get path for alias", command: "goto -x myproject" },
            Example { description: "Use in command", command: "ls $(goto -x proj)" },
        ],
        see_also: &["--list"],
        category: CommandCategory::Advanced,
        essential: false,
    },
    CommandHelp {
        short_flag: None,
        long_flag: "--install",
        description: "Install shell integration",
        detailed: "Sets up the goto shell function for your shell. This enables the \
                   actual directory changing functionality.",
        usage: "goto --install [--shell=<shell>] [--skip-rc] [--dry-run]",
        options: &[
            CommandOption { flag: "--shell=<shell>", description: "Shell type: bash, zsh, fish" },
            CommandOption { flag: "--skip-rc", description: "Don't modify shell rc file" },
            CommandOption { flag: "--dry-run", description: "Show what would be done" },
        ],
        examples: &[
            Example { description: "Install for current shell", command: "goto --install" },
            Example { description: "Install for specific shell", command: "goto --install --shell=zsh" },
            Example { description: "Preview installation", command: "goto --install --dry-run" },
        ],
        see_also: &["--version"],
        category: CommandCategory::Advanced,
        essential: false,
    },
    CommandHelp {
        short_flag: None,
        long_flag: "--prune-snooze",
        description: "Snooze stale alias notifications",
        detailed: "Temporarily suppresses notifications about aliases pointing to \
                   non-existent directories for the specified number of days.",
        usage: "goto --prune-snooze <days>",
        options: &[],
        examples: &[
            Example { description: "Snooze for 7 days", command: "goto --prune-snooze 7" },
        ],
        see_also: &["--cleanup"],
        category: CommandCategory::Advanced,
        essential: false,
    },
    CommandHelp {
        short_flag: Some("-U"),
        long_flag: "--update",
        description: "Update goto to the latest version",
        detailed: "Downloads and installs the latest version of goto from GitHub.",
        usage: "goto -U",
        options: &[],
        examples: &[
            Example { description: "Update to latest version", command: "goto -U" },
        ],
        see_also: &["--check-update", "--version"],
        category: CommandCategory::Advanced,
        essential: false,
    },
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
    fn test_print_command_help_not_found() {
        let result = print_command_help("nonexistent");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Unknown command"));
        assert!(err.contains("--help-all"));
    }

    #[test]
    fn test_all_commands_have_required_fields() {
        for cmd in COMMANDS {
            assert!(!cmd.description.is_empty(), "{} missing description", cmd.long_flag);
            assert!(!cmd.usage.is_empty(), "{} missing usage", cmd.long_flag);
            assert!(!cmd.examples.is_empty(), "{} needs at least one example", cmd.long_flag);
        }
    }

    #[test]
    fn test_essential_commands_count() {
        let essential_count = COMMANDS.iter().filter(|c| c.essential).count();
        assert!(essential_count >= 4, "Brief help should show at least 4 commands, got {}", essential_count);
        assert!(essential_count <= 5, "Brief help should show at most 5 commands, got {}", essential_count);
    }

    #[test]
    fn test_placeholder_format_consistency() {
        for cmd in COMMANDS {
            // Usage should use <placeholder> format for required arguments
            // Check that we use <alias> not bare "alias" (except in command descriptions)
            if cmd.usage.contains(" alias ") && !cmd.usage.contains("<alias>") {
                panic!("{} usage should use <placeholder> format: {}", cmd.long_flag, cmd.usage);
            }
        }
    }

    #[test]
    fn test_all_categories_have_commands() {
        for category in [CommandCategory::Essential, CommandCategory::Management,
                         CommandCategory::Info, CommandCategory::Advanced] {
            let count = COMMANDS.iter().filter(|c| c.category == category).count();
            assert!(count > 0, "{:?} category has no commands", category);
        }
    }

    #[test]
    fn test_find_command_by_long_flag() {
        let cmd = find_command("--register");
        assert!(cmd.is_some(), "--register should be findable");
        assert_eq!(cmd.unwrap().long_flag, "--register");
    }

    #[test]
    fn test_find_command_by_short_flag() {
        let cmd = find_command("-r");
        assert!(cmd.is_some(), "-r should be findable");
        assert_eq!(cmd.unwrap().long_flag, "--register");
    }

    #[test]
    fn test_find_command_by_name() {
        let cmd = find_command("register");
        assert!(cmd.is_some(), "register should be findable");
        assert_eq!(cmd.unwrap().long_flag, "--register");
    }

    #[test]
    fn test_commands_have_see_also() {
        // Most commands should have related commands
        let commands_with_see_also = COMMANDS.iter().filter(|c| !c.see_also.is_empty()).count();
        assert!(
            commands_with_see_also >= COMMANDS.len() / 2,
            "At least half of commands should have see_also references"
        );
    }

    #[test]
    fn test_essential_commands_are_marked() {
        // Check that the core commands are marked essential
        let essential_flags: Vec<&str> = COMMANDS
            .iter()
            .filter(|c| c.essential)
            .map(|c| c.long_flag)
            .collect();

        assert!(essential_flags.contains(&"--register"), "--register should be essential");
        assert!(essential_flags.contains(&"--list"), "--list should be essential");
        assert!(essential_flags.contains(&"--tags"), "--tags should be essential");
    }

    #[test]
    fn test_command_registry_not_empty() {
        assert!(!COMMANDS.is_empty(), "COMMANDS registry should not be empty");
        assert!(COMMANDS.len() >= 20, "Should have at least 20 commands, got {}", COMMANDS.len());
    }
}
