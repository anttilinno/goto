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
            println!("goto version {}", cli::version());
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

    let mut db = Database::load(&config).map_err(|e| {
        eprintln!("Error loading database: {}", e);
        5u8
    })?;

    match parsed.command {
        Command::Help | Command::Version | Command::Config => unreachable!(),

        Command::List { sort, filter } => {
            commands::list::list_with_options(&db, &config, sort.as_deref(), filter.as_deref())
                .map_err(handle_error)
        }

        Command::ListNames => commands::list::list_names(&db).map_err(handle_error),

        Command::ListTagsRaw => commands::tags::list_tags_raw(&db).map_err(handle_error),

        Command::Stats => commands::stats::stats(&db).map_err(handle_error),

        Command::Register { name, path, tags } => {
            commands::register::register_with_tags(&mut db, &name, &path, &tags)
                .map_err(handle_error)
        }

        Command::Unregister { name } => {
            commands::register::unregister(&mut db, &name).map_err(handle_error)
        }

        Command::Expand { alias } => commands::navigate::expand(&db, &alias).map_err(handle_error),

        Command::Cleanup { dry_run } => {
            commands::cleanup::cleanup(&mut db, dry_run).map_err(handle_error)
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

        Command::ListTags => commands::tags::list_tags(&db).map_err(handle_error),

        Command::Recent { count, navigate_to } => {
            if let Some(n) = navigate_to {
                commands::stats::navigate_to_recent(&mut db, n).map_err(handle_error)
            } else {
                commands::stats::show_recent(&db, count.unwrap_or(10)).map_err(handle_error)
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
            commands::navigate::navigate(&mut db, &alias).map_err(handle_error)
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
