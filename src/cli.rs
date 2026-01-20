//! Command-line argument parsing for goto

use crate::commands::import_export::ImportStrategy;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parsed command-line arguments
#[derive(Debug)]
pub struct Args {
    pub command: Command,
}

/// All supported commands
#[derive(Debug)]
pub enum Command {
    Help,
    Version,
    Config,
    List {
        sort: Option<String>,
        filter: Option<String>,
    },
    ListNames,
    Register {
        name: String,
        path: String,
        tags: Vec<String>,
    },
    Unregister {
        name: String,
    },
    Navigate {
        alias: String,
    },
    Expand {
        alias: String,
    },
    Cleanup {
        dry_run: bool,
    },
    Push {
        alias: String,
    },
    Pop,
    Rename {
        old_name: String,
        new_name: String,
    },
    Tag {
        alias: String,
        tag: String,
    },
    Untag {
        alias: String,
        tag: String,
    },
    ListTags,
    ListTagsRaw,
    Stats,
    Recent {
        count: Option<usize>,
        navigate_to: Option<usize>,
    },
    RecentClear,
    Export,
    Import {
        file: String,
        strategy: ImportStrategy,
    },
    Install {
        shell: Option<String>,
        skip_rc: bool,
        dry_run: bool,
    },
}

/// Parse command-line arguments into a structured Args object
pub fn parse_args(args: &[String]) -> Result<Args, String> {
    if args.len() < 2 {
        return Err("No arguments provided".to_string());
    }

    let arg = &args[1];
    let command = match arg.as_str() {
        "-h" | "--help" => Command::Help,

        "-v" | "--version" => Command::Version,

        "--config" => Command::Config,

        "-l" | "--list" => Command::List {
            sort: find_flag_value(args, "--sort="),
            filter: find_flag_value(args, "--filter="),
        },

        "--stats" => Command::Stats,

        "--list-aliases" | "--names-only" => Command::ListNames,

        "--tags-raw" => Command::ListTagsRaw,

        "-r" | "--register" => {
            if args.len() < 4 {
                return Err("Usage: goto -r <alias> <directory> [--tags=tag1,tag2]".to_string());
            }
            let tags = find_flag_value(args, "--tags=")
                .map(|t| t.split(',').map(String::from).collect::<Vec<_>>())
                .unwrap_or_default();
            Command::Register {
                name: args[2].clone(),
                path: args[3].clone(),
                tags,
            }
        }

        "-u" | "--unregister" => {
            if args.len() < 3 {
                return Err("Usage: goto -u <alias>".to_string());
            }
            Command::Unregister {
                name: args[2].clone(),
            }
        }

        "-x" | "--expand" => {
            if args.len() < 3 {
                return Err("Usage: goto -x <alias>".to_string());
            }
            Command::Expand {
                alias: args[2].clone(),
            }
        }

        "-c" | "--cleanup" => Command::Cleanup {
            dry_run: args.iter().any(|a| a == "--dry-run"),
        },

        "-p" | "--push" => {
            if args.len() < 3 {
                return Err("Usage: goto -p <alias>".to_string());
            }
            Command::Push {
                alias: args[2].clone(),
            }
        }

        "-o" | "--pop" => Command::Pop,

        "--export" => Command::Export,

        "--rename" => {
            if args.len() < 4 {
                return Err("Usage: goto --rename <old-alias> <new-alias>".to_string());
            }
            Command::Rename {
                old_name: args[2].clone(),
                new_name: args[3].clone(),
            }
        }

        "--tag" => {
            if args.len() < 4 {
                return Err("Usage: goto --tag <alias> <tag>".to_string());
            }
            Command::Tag {
                alias: args[2].clone(),
                tag: args[3].clone(),
            }
        }

        "--untag" => {
            if args.len() < 4 {
                return Err("Usage: goto --untag <alias> <tag>".to_string());
            }
            Command::Untag {
                alias: args[2].clone(),
                tag: args[3].clone(),
            }
        }

        "--tags" => Command::ListTags,

        "--recent" => {
            if args.len() >= 3 {
                if let Ok(n) = args[2].parse::<usize>() {
                    if n >= 1 && n <= 20 && args.len() == 3 {
                        return Ok(Args {
                            command: Command::Recent {
                                count: None,
                                navigate_to: Some(n),
                            },
                        });
                    } else {
                        return Ok(Args {
                            command: Command::Recent {
                                count: Some(n),
                                navigate_to: None,
                            },
                        });
                    }
                }
            }
            Command::Recent {
                count: Some(10),
                navigate_to: None,
            }
        }

        "--recent-clear" => Command::RecentClear,

        "--import" => {
            if args.len() < 3 {
                return Err(
                    "Usage: goto --import <file> [--strategy=skip|overwrite|rename]".to_string(),
                );
            }
            let strategy_str = find_flag_value(args, "--strategy=").unwrap_or_else(|| "skip".to_string());
            let strategy = ImportStrategy::from_str(&strategy_str)
                .map_err(|e| e.to_string())?;
            Command::Import {
                file: args[2].clone(),
                strategy,
            }
        }

        "--install" => Command::Install {
            shell: find_flag_value(args, "--shell="),
            skip_rc: args.iter().any(|a| a == "--skip-rc"),
            dry_run: args.iter().any(|a| a == "--dry-run"),
        },

        _ => {
            if arg.starts_with('-') {
                return Err(format!("Unknown option: {}", arg));
            }
            // Default action: navigate to alias
            Command::Navigate {
                alias: arg.clone(),
            }
        }
    };

    Ok(Args { command })
}

/// Find a flag value with the given prefix (e.g., "--sort=alpha")
fn find_flag_value(args: &[String], prefix: &str) -> Option<String> {
    args.iter()
        .find(|a| a.starts_with(prefix))
        .map(|a| a[prefix.len()..].to_string())
}

/// Print brief usage information
pub fn print_usage() {
    println!("Usage: goto <alias> or goto [OPTIONS]");
    println!("Try 'goto --help' for more information.");
}

/// Print the full help text
pub fn print_help() {
    print!(
        r#"goto - Navigate to aliased directories

Usage:
  goto <alias>                    Navigate to the directory
  goto -r <alias> <directory>     Register a new alias
  goto -r <alias> <dir> --tags=   Register with tags
  goto -u <alias>                 Unregister an alias
  goto -l                         List all aliases
  goto -l --sort=<order>          List aliases with sorting
  goto -l --filter=<tag>          List aliases with tag
  goto -x <alias>                 Expand alias to path
  goto -c                         Cleanup invalid aliases
  goto -c --dry-run               List invalid aliases (don't remove)
  goto -p <alias>                 Push current dir, goto alias
  goto -o                         Pop and return to directory
  goto --rename <old> <new>       Rename an alias
  goto --tag <alias> <tag>        Add tag to alias
  goto --untag <alias> <tag>      Remove tag from alias
  goto --tags                     List all tags with counts
  goto --stats                    Show usage statistics
  goto --recent                   List recently visited directories
  goto --recent <N>               Navigate to Nth most recent
  goto --recent-clear             Clear recent history
  goto --export                   Export aliases to TOML (stdout)
  goto --import <file>            Import aliases from TOML file
  goto --config                   Show current configuration
  goto --install                  Install shell integration
  goto -v                         Show version
  goto -h                         Show this help

Sort options (use with -l/--list):
  --sort=alpha                    Sort alphabetically (default)
  --sort=usage                    Sort by use count (most used first)
  --sort=recent                   Sort by last used (most recent first)

Filter options (use with -l/--list):
  --filter=<tag>                  Show only aliases with tag

Import strategies (use with --import):
  --strategy=skip                 Skip existing aliases (default)
  --strategy=overwrite            Overwrite existing aliases
  --strategy=rename               Rename conflicting aliases (add suffix)

Install options (use with --install):
  --shell=bash|zsh|fish           Shell to configure (auto-detects from $SHELL)
  --skip-rc                       Don't modify shell rc file
  --dry-run                       Show what would be done without making changes

Tag rules:
  - Tags are case-insensitive (stored lowercase)
  - Tags must be alphanumeric with dash/underscore
  - No spaces in tags

Examples:
  goto -r dev ~/Development       Register 'dev' alias
  goto -r proj ~/code --tags=work,go  Register with tags
  goto dev                        Navigate to ~/Development
  goto -l --sort=usage            List aliases by usage
  goto -l --filter=work           List aliases tagged 'work'
  goto --tag dev golang           Add 'golang' tag to 'dev'
  goto --untag dev golang         Remove 'golang' tag from 'dev'
  goto --tags                     List all tags with counts
  goto --stats                    Show usage statistics
  goto --recent                   Show recently visited aliases
  goto --recent 3                 Navigate to 3rd most recent
  goto -p work                    Save location, go to 'work'
  goto -o                         Return to saved location
  goto --export > backup.toml     Backup aliases to file
  goto --import backup.toml       Restore aliases from backup
"#
    );
}

/// Get the version string
pub fn version() -> &'static str {
    VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_parse_help() {
        let result = parse_args(&args(&["goto", "-h"]));
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().command, Command::Help));
    }

    #[test]
    fn test_parse_version() {
        let result = parse_args(&args(&["goto", "--version"]));
        assert!(result.is_ok());
        assert!(matches!(result.unwrap().command, Command::Version));
    }

    #[test]
    fn test_parse_navigate() {
        let result = parse_args(&args(&["goto", "myalias"]));
        assert!(result.is_ok());
        if let Command::Navigate { alias } = result.unwrap().command {
            assert_eq!(alias, "myalias");
        } else {
            panic!("Expected Navigate command");
        }
    }

    #[test]
    fn test_parse_register() {
        let result = parse_args(&args(&["goto", "-r", "dev", "/path/to/dev"]));
        assert!(result.is_ok());
        if let Command::Register { name, path, tags } = result.unwrap().command {
            assert_eq!(name, "dev");
            assert_eq!(path, "/path/to/dev");
            assert!(tags.is_empty());
        } else {
            panic!("Expected Register command");
        }
    }

    #[test]
    fn test_parse_register_with_tags() {
        let result = parse_args(&args(&["goto", "-r", "dev", "/path", "--tags=work,rust"]));
        assert!(result.is_ok());
        if let Command::Register { name, path, tags } = result.unwrap().command {
            assert_eq!(name, "dev");
            assert_eq!(path, "/path");
            assert_eq!(tags, vec!["work", "rust"]);
        } else {
            panic!("Expected Register command");
        }
    }

    #[test]
    fn test_parse_cleanup_dry_run() {
        let result = parse_args(&args(&["goto", "-c", "--dry-run"]));
        assert!(result.is_ok());
        if let Command::Cleanup { dry_run } = result.unwrap().command {
            assert!(dry_run);
        } else {
            panic!("Expected Cleanup command");
        }
    }

    #[test]
    fn test_parse_cleanup_no_dry_run() {
        let result = parse_args(&args(&["goto", "--cleanup"]));
        assert!(result.is_ok());
        if let Command::Cleanup { dry_run } = result.unwrap().command {
            assert!(!dry_run);
        } else {
            panic!("Expected Cleanup command");
        }
    }

    #[test]
    fn test_parse_list_with_options() {
        let result = parse_args(&args(&["goto", "-l", "--sort=usage", "--filter=work"]));
        assert!(result.is_ok());
        if let Command::List { sort, filter } = result.unwrap().command {
            assert_eq!(sort, Some("usage".to_string()));
            assert_eq!(filter, Some("work".to_string()));
        } else {
            panic!("Expected List command");
        }
    }

    #[test]
    fn test_parse_unknown_option() {
        let result = parse_args(&args(&["goto", "--unknown"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown option"));
    }

    #[test]
    fn test_parse_no_args() {
        let result = parse_args(&args(&["goto"]));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_register_missing_args() {
        let result = parse_args(&args(&["goto", "-r", "dev"]));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Usage:"));
    }
}
