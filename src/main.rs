//! goto - CLI entry point for the goto directory navigation tool

use std::env;
use std::process::ExitCode;

use goto::cli::{self, Command};
use goto::commands;
use goto::config::Config;
use goto::database::Database;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code),
    }
}

fn run() -> Result<(), u8> {
    let args: Vec<String> = env::args().collect();

    let parsed = match cli::parse_args(&args) {
        Ok(args) => args,
        Err(msg) => {
            eprintln!("{}", msg);
            cli::print_usage();
            return Err(1);
        }
    };

    // Handle commands that don't need config/database
    match &parsed.command {
        Command::Help => {
            cli::print_help();
            return Ok(());
        }
        Command::Version => {
            // Try to show version with update status if config is available
            if let Ok(config) = Config::load() {
                println!("{}", commands::update::version_with_update_status(&config));
            } else {
                println!("goto version {}", cli::version());
            }
            return Ok(());
        }
        Command::Install { shell, skip_rc, dry_run } => {
            use commands::install::{InstallOptions, ShellType};

            let shell_type = match shell {
                Some(s) => ShellType::from_str(s).map_err(|e| {
                    eprintln!("{}", e);
                    3u8
                })?,
                None => ShellType::detect().map_err(|e| {
                    eprintln!("{}", e);
                    3u8
                })?,
            };

            let mut options = InstallOptions::new(shell_type);
            options.skip_rc = *skip_rc;
            options.dry_run = *dry_run;

            commands::install::install(&options).map_err(|e| {
                eprintln!("{}", e);
                5u8
            })?;
            return Ok(());
        }
        _ => {}
    }

    let config = Config::load().map_err(|e| {
        eprintln!("Error loading config: {}", e);
        5u8
    })?;

    // Handle config command (needs config but not database)
    if matches!(parsed.command, Command::Config) {
        commands::config::show_config(&config);
        return Ok(());
    }

    // Handle update commands
    match &parsed.command {
        Command::Update => {
            commands::update::perform_update(&config).map_err(|e| {
                eprintln!("{}", e);
                5u8
            })?;
            return Ok(());
        }
        Command::CheckUpdate => {
            match commands::update::check_for_updates(&config, true) {
                Ok(Some(version)) => {
                    println!(
                        "Update available: {} (current: {})",
                        version,
                        commands::update::current_version()
                    );
                    println!("Run 'goto --update' to upgrade.");
                }
                Ok(None) => {
                    println!(
                        "You are running the latest version ({}).",
                        commands::update::current_version()
                    );
                }
                Err(e) => {
                    eprintln!("Failed to check for updates: {}", e);
                    return Err(5);
                }
            }
            return Ok(());
        }
        _ => {}
    }

    let mut db = Database::load(&config).map_err(|e| {
        eprintln!("Error loading database: {}", e);
        5u8
    })?;

    match parsed.command {
        Command::Help | Command::Version | Command::Config | Command::Install { .. }
        | Command::Update | Command::CheckUpdate => unreachable!(),

        Command::List { sort, filter } => {
            commands::list::list_with_options(&db, &config, sort.as_deref(), filter.as_deref())
                .map_err(handle_error)
        }

        Command::ListNames => commands::list::list_names(&db).map_err(handle_error),

        Command::ListTagsRaw => commands::tags::list_tags_raw(&db).map_err(handle_error),

        Command::Stats => commands::stats::stats(&db, &config).map_err(handle_error),

        Command::Register { name, path, tags } => {
            commands::register::register_with_tags(&mut db, &name, &path, &tags)
                .map_err(handle_error)
        }

        Command::Unregister { name } => {
            commands::register::unregister(&mut db, &name).map_err(handle_error)
        }

        Command::Expand { alias } => commands::navigate::expand(&db, &alias).map_err(handle_error),

        Command::Cleanup { dry_run } => {
            commands::cleanup::cleanup(&mut db, &config, dry_run).map_err(handle_error)
        }

        Command::Push { alias } => {
            commands::stack::push(&config, &mut db, &alias).map_err(handle_error)
        }

        Command::Pop => commands::stack::pop(&config).map_err(handle_error),

        Command::Rename { old_name, new_name } => {
            commands::register::rename(&mut db, &old_name, &new_name).map_err(handle_error)
        }

        Command::Tag { alias, tag } => {
            commands::tags::tag(&mut db, &alias, &tag).map_err(handle_error)
        }

        Command::Untag { alias, tag } => {
            commands::tags::untag(&mut db, &alias, &tag).map_err(handle_error)
        }

        Command::ListTags => commands::tags::list_tags(&db, &config).map_err(handle_error),

        Command::Recent { count, navigate_to } => {
            if let Some(n) = navigate_to {
                commands::stats::navigate_to_recent(&mut db, n).map_err(handle_error)
            } else {
                commands::stats::show_recent(&db, &config, count.unwrap_or(10)).map_err(handle_error)
            }
        }

        Command::RecentClear => commands::stats::clear_recent(&mut db).map_err(handle_error),

        Command::Export => commands::import_export::export(&db).map_err(handle_error),

        Command::Import { file, strategy } => {
            match commands::import_export::import(&mut db, &file, strategy) {
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

        Command::Navigate { alias } => {
            let result = commands::navigate::navigate(&mut db, &alias).map_err(handle_error);
            // Show update notification after successful navigation (goes to stderr)
            if result.is_ok() {
                commands::update::notify_if_update_available(&config);
            }
            result
        }
    }
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
