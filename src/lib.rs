//! goto - Navigate to aliased directories with autocomplete support
//!
//! This library provides functionality for managing directory aliases,
//! enabling quick navigation between frequently used directories.

pub mod alias;
pub mod cli;
pub mod commands;
pub mod config;
pub mod database;
pub mod fuzzy;
pub mod stack;

pub use alias::Alias;
pub use cli::{parse_args, Args, Command};
pub use config::Config;
pub use database::Database;
pub use stack::Stack;
