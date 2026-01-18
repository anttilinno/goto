//! goto - CLI entry point for the goto directory navigation tool

use std::env;
use std::process::ExitCode;

use goto::commands;
use goto::config::Config;
use goto::database::Database;

const VERSION: &str = "1.0.0";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code),
    }
}

fn run() -> Result<(), u8> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Err(1);
    }

    let config = Config::load().map_err(|e| {
        eprintln!("Error loading config: {}", e);
        5u8
    })?;

    let mut db = Database::load(&config).map_err(|e| {
        eprintln!("Error loading database: {}", e);
        5u8
    })?;

    let arg = &args[1];

    match arg.as_str() {
        "-h" | "--help" => {
            print_help();
            Ok(())
        }

        "-v" | "--version" => {
            println!("goto version {}", VERSION);
            Ok(())
        }

        "--config" => {
            commands::config::show_config(&config);
            Ok(())
        }

        "-l" | "--list" => {
            let sort_order = find_flag_value(&args, "--sort=");
            let filter_tag = find_flag_value(&args, "--filter=");
            commands::list::list_with_options(&db, &config, sort_order.as_deref(), filter_tag.as_deref())
                .map_err(handle_error)
        }

        "--stats" => {
            commands::stats::stats(&db).map_err(handle_error)
        }

        "--list-aliases" | "--names-only" => {
            commands::list::list_names(&db).map_err(handle_error)
        }

        "--tags-raw" => {
            commands::tags::list_tags_raw(&db).map_err(handle_error)
        }

        "-r" | "--register" => {
            if args.len() < 4 {
                eprintln!("Usage: goto -r <alias> <directory> [--tags=tag1,tag2]");
                return Err(1);
            }
            let tags = find_flag_value(&args, "--tags=")
                .map(|t| t.split(',').map(String::from).collect::<Vec<_>>())
                .unwrap_or_default();
            commands::register::register_with_tags(&mut db, &args[2], &args[3], &tags)
                .map_err(handle_error)
        }

        "-u" | "--unregister" => {
            if args.len() < 3 {
                eprintln!("Usage: goto -u <alias>");
                return Err(1);
            }
            commands::register::unregister(&mut db, &args[2]).map_err(handle_error)
        }

        "-x" | "--expand" => {
            if args.len() < 3 {
                eprintln!("Usage: goto -x <alias>");
                return Err(1);
            }
            commands::navigate::expand(&db, &args[2]).map_err(handle_error)
        }

        "-c" | "--cleanup" => {
            commands::cleanup::cleanup(&mut db).map_err(handle_error)
        }

        "-p" | "--push" => {
            if args.len() < 3 {
                eprintln!("Usage: goto -p <alias>");
                return Err(1);
            }
            commands::stack::push(&config, &mut db, &args[2]).map_err(handle_error)
        }

        "-o" | "--pop" => {
            commands::stack::pop(&config).map_err(handle_error)
        }

        "--export" => {
            commands::import_export::export(&db).map_err(handle_error)
        }

        "--rename" => {
            if args.len() < 4 {
                eprintln!("Usage: goto --rename <old-alias> <new-alias>");
                return Err(1);
            }
            commands::register::rename(&mut db, &args[2], &args[3]).map_err(handle_error)
        }

        "--tag" => {
            if args.len() < 4 {
                eprintln!("Usage: goto --tag <alias> <tag>");
                return Err(1);
            }
            commands::tags::tag(&mut db, &args[2], &args[3]).map_err(handle_error)
        }

        "--untag" => {
            if args.len() < 4 {
                eprintln!("Usage: goto --untag <alias> <tag>");
                return Err(1);
            }
            commands::tags::untag(&mut db, &args[2], &args[3]).map_err(handle_error)
        }

        "--tags" => {
            commands::tags::list_tags(&db).map_err(handle_error)
        }

        "--recent" => {
            if args.len() >= 3 {
                if let Ok(n) = args[2].parse::<usize>() {
                    if n >= 1 && n <= 20 && args.len() == 3 {
                        // Single small number: navigate to Nth recent
                        return commands::stats::navigate_to_recent(&mut db, n)
                            .map_err(handle_error);
                    } else {
                        // Show as list with limit
                        return commands::stats::show_recent(&db, n).map_err(handle_error);
                    }
                }
            }
            commands::stats::show_recent(&db, 10).map_err(handle_error)
        }

        "--recent-clear" => {
            commands::stats::clear_recent(&mut db).map_err(handle_error)
        }

        "--import" => {
            if args.len() < 3 {
                eprintln!("Usage: goto --import <file> [--strategy=skip|overwrite|rename]");
                return Err(1);
            }
            let strategy_str = find_flag_value(&args, "--strategy=").unwrap_or_else(|| "skip".to_string());
            let strategy = commands::import_export::ImportStrategy::from_str(&strategy_str)
                .map_err(|e| { eprintln!("{}", e); 1u8 })?;

            match commands::import_export::import(&mut db, &args[2], strategy) {
                Ok(result) => {
                    for warning in &result.warnings {
                        eprintln!("{}", warning);
                    }
                    print!("Import complete: {} imported", result.imported);
                    if result.skipped > 0 {
                        print!(", {} skipped", result.skipped);
                    }
                    if result.renamed > 0 {
                        print!(", {} renamed", result.renamed);
                    }
                    println!();
                    Ok(())
                }
                Err(e) => Err(handle_error(e)),
            }
        }

        _ => {
            if arg.starts_with('-') {
                eprintln!("Unknown option: {}", arg);
                return Err(1);
            }
            // Default action: navigate to alias
            commands::navigate::navigate(&mut db, arg).map_err(handle_error)
        }
    }
}

fn find_flag_value(args: &[String], prefix: &str) -> Option<String> {
    args.iter()
        .find(|a| a.starts_with(prefix))
        .map(|a| a[prefix.len()..].to_string())
}

fn handle_error(err: Box<dyn std::error::Error>) -> u8 {
    eprintln!("{}", err);

    // Map error types to exit codes
    let err_str = err.to_string();
    if err_str.contains("directory does not exist") {
        2
    } else if err_str.contains("invalid alias") || err_str.contains("invalid tag") {
        3
    } else if err_str.contains("already exists") {
        4
    } else if err_str.contains("not found") || err_str.contains("stack is empty") {
        1
    } else {
        5
    }
}

fn print_usage() {
    println!("Usage: goto <alias> or goto [OPTIONS]");
    println!("Try 'goto --help' for more information.");
}

fn print_help() {
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
