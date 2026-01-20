//! Install and uninstall commands for goto

use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

/// Shell wrapper script for bash (embedded)
const SHELL_BASH: &str = include_str!("../../shell/goto.bash");

/// Shell wrapper script for zsh (embedded)
const SHELL_ZSH: &str = include_str!("../../shell/goto.zsh");

/// Shell wrapper script for fish (embedded)
const SHELL_FISH: &str = include_str!("../../shell/goto.fish");

/// Supported shell types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
}

impl ShellType {
    /// Parse shell type from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(ShellType::Bash),
            "zsh" => Ok(ShellType::Zsh),
            "fish" => Ok(ShellType::Fish),
            _ => Err(format!(
                "Invalid shell type '{}'. Must be bash, zsh, or fish.",
                s
            )),
        }
    }

    /// Auto-detect shell from SHELL environment variable
    pub fn detect() -> Result<Self, String> {
        let shell = env::var("SHELL").unwrap_or_default();
        let shell_name = shell.rsplit('/').next().unwrap_or("");

        match shell_name {
            "bash" => Ok(ShellType::Bash),
            "zsh" => Ok(ShellType::Zsh),
            "fish" => Ok(ShellType::Fish),
            _ => Err(format!(
                "Could not auto-detect shell from '{}'. Please specify --shell=bash|zsh|fish",
                shell
            )),
        }
    }

    /// Get the shell wrapper script content
    fn wrapper_content(&self) -> &'static str {
        match self {
            ShellType::Bash => SHELL_BASH,
            ShellType::Zsh => SHELL_ZSH,
            ShellType::Fish => SHELL_FISH,
        }
    }

    /// Get the wrapper filename
    fn wrapper_filename(&self) -> &'static str {
        match self {
            ShellType::Bash => "goto.bash",
            ShellType::Zsh => "goto.zsh",
            ShellType::Fish => "goto.fish",
        }
    }

    /// Get the rc file path
    fn rc_file(&self) -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        match self {
            ShellType::Bash => PathBuf::from(home).join(".bashrc"),
            ShellType::Zsh => PathBuf::from(home).join(".zshrc"),
            ShellType::Fish => PathBuf::from(home)
                .join(".config")
                .join("fish")
                .join("config.fish"),
        }
    }
}

/// Install options
pub struct InstallOptions {
    pub shell: ShellType,
    pub skip_rc: bool,
    pub dry_run: bool,
}

impl InstallOptions {
    /// Create with defaults
    pub fn new(shell: ShellType) -> Self {
        Self {
            shell,
            skip_rc: false,
            dry_run: false,
        }
    }
}

/// Install shell integration (wrapper script + rc file modification)
pub fn install(options: &InstallOptions) -> Result<(), Box<dyn Error>> {
    let home = env::var("HOME")?;
    let config_dir = PathBuf::from(&home).join(".config").join("goto");
    let wrapper_path = config_dir.join(options.shell.wrapper_filename());
    let rc_file = options.shell.rc_file();
    let source_line = format!("source {}", wrapper_path.display());

    println!("Installing goto shell integration for {:?}...", options.shell);
    println!();

    // Step 1: Create config directory and copy shell wrapper
    println!("[1/2] Installing shell wrapper to {}", wrapper_path.display());
    if options.dry_run {
        println!("  Would create: {}", config_dir.display());
        println!("  Would write: {}", wrapper_path.display());
    } else {
        fs::create_dir_all(&config_dir)?;
        fs::write(&wrapper_path, options.shell.wrapper_content())?;
        println!("  Installed");
    }

    // Step 2: Update shell config (unless skipped)
    if options.skip_rc {
        println!("[2/2] Skipping rc file modification (--skip-rc)");
        println!("  Add this line to your shell config manually:");
        println!("  {}", source_line);
    } else {
        println!("[2/2] Updating {}", rc_file.display());
        let rc_content = fs::read_to_string(&rc_file).unwrap_or_default();
        let already_present = rc_content.contains(&source_line);

        if options.dry_run {
            if already_present {
                println!("  Source line already present, would skip");
            } else {
                println!("  Would append: {}", source_line);
            }
        } else {
            if already_present {
                println!("  Source line already present, skipping");
            } else {
                // Create parent directory if needed (for fish)
                if let Some(parent) = rc_file.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut content = rc_content;
                content.push_str("\n# goto - directory navigation\n");
                content.push_str(&source_line);
                content.push('\n');
                fs::write(&rc_file, content)?;
                println!("  Added source line");
            }
        }
    }

    println!();
    if options.dry_run {
        println!("Dry run complete. No changes were made.");
    } else {
        println!("Installation complete!");
        println!("Restart your shell or run: source {}", rc_file.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_type_from_str() {
        assert!(matches!(ShellType::from_str("bash"), Ok(ShellType::Bash)));
        assert!(matches!(ShellType::from_str("ZSH"), Ok(ShellType::Zsh)));
        assert!(matches!(ShellType::from_str("Fish"), Ok(ShellType::Fish)));
        assert!(ShellType::from_str("invalid").is_err());
    }

    #[test]
    fn test_shell_type_from_str_case_insensitive() {
        // Test various case combinations
        assert!(matches!(ShellType::from_str("BASH"), Ok(ShellType::Bash)));
        assert!(matches!(ShellType::from_str("Bash"), Ok(ShellType::Bash)));
        assert!(matches!(ShellType::from_str("zsh"), Ok(ShellType::Zsh)));
        assert!(matches!(ShellType::from_str("ZSH"), Ok(ShellType::Zsh)));
        assert!(matches!(ShellType::from_str("fish"), Ok(ShellType::Fish)));
        assert!(matches!(ShellType::from_str("FISH"), Ok(ShellType::Fish)));
    }

    #[test]
    fn test_shell_type_from_str_invalid() {
        let result = ShellType::from_str("invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid shell type"));
        assert!(err.contains("invalid"));
        assert!(err.contains("bash, zsh, or fish"));
    }

    #[test]
    fn test_shell_type_from_str_empty() {
        let result = ShellType::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_shell_type_from_str_similar_names() {
        // Test that partial/similar names don't match
        assert!(ShellType::from_str("bas").is_err());
        assert!(ShellType::from_str("bashy").is_err());
        assert!(ShellType::from_str("zshell").is_err());
        assert!(ShellType::from_str("fishy").is_err());
    }

    #[test]
    fn test_wrapper_content_not_empty() {
        assert!(!ShellType::Bash.wrapper_content().is_empty());
        assert!(!ShellType::Zsh.wrapper_content().is_empty());
        assert!(!ShellType::Fish.wrapper_content().is_empty());
    }

    #[test]
    fn test_wrapper_content_contains_goto() {
        // Verify wrapper scripts contain expected goto function/alias
        assert!(ShellType::Bash.wrapper_content().contains("goto"));
        assert!(ShellType::Zsh.wrapper_content().contains("goto"));
        assert!(ShellType::Fish.wrapper_content().contains("goto"));
    }

    #[test]
    fn test_wrapper_filename() {
        assert_eq!(ShellType::Bash.wrapper_filename(), "goto.bash");
        assert_eq!(ShellType::Zsh.wrapper_filename(), "goto.zsh");
        assert_eq!(ShellType::Fish.wrapper_filename(), "goto.fish");
    }

    #[test]
    fn test_rc_file_bash() {
        let shell = ShellType::Bash;
        let rc = shell.rc_file();
        assert!(rc.to_string_lossy().ends_with(".bashrc"));
    }

    #[test]
    fn test_rc_file_zsh() {
        let shell = ShellType::Zsh;
        let rc = shell.rc_file();
        assert!(rc.to_string_lossy().ends_with(".zshrc"));
    }

    #[test]
    fn test_rc_file_fish() {
        let shell = ShellType::Fish;
        let rc = shell.rc_file();
        let rc_str = rc.to_string_lossy();
        assert!(rc_str.contains("fish"));
        assert!(rc_str.ends_with("config.fish"));
    }

    #[test]
    fn test_rc_file_fish_path_structure() {
        let shell = ShellType::Fish;
        let rc = shell.rc_file();
        // Fish config should be in .config/fish/config.fish
        let components: Vec<_> = rc.components().collect();
        let path_str = rc.to_string_lossy();
        assert!(path_str.contains(".config"));
        assert!(path_str.contains("fish"));
        // Should have at least 4 components: home, .config, fish, config.fish
        assert!(components.len() >= 4);
    }

    #[test]
    fn test_install_options_new_defaults() {
        let opts = InstallOptions::new(ShellType::Bash);
        assert_eq!(opts.shell, ShellType::Bash);
        assert!(!opts.dry_run);
        assert!(!opts.skip_rc);
    }

    #[test]
    fn test_install_options_new_all_shells() {
        // Test InstallOptions::new works for all shell types
        let bash_opts = InstallOptions::new(ShellType::Bash);
        assert_eq!(bash_opts.shell, ShellType::Bash);

        let zsh_opts = InstallOptions::new(ShellType::Zsh);
        assert_eq!(zsh_opts.shell, ShellType::Zsh);

        let fish_opts = InstallOptions::new(ShellType::Fish);
        assert_eq!(fish_opts.shell, ShellType::Fish);
    }

    #[test]
    fn test_install_options_modify_flags() {
        let mut opts = InstallOptions::new(ShellType::Zsh);
        opts.dry_run = true;
        opts.skip_rc = true;
        assert!(opts.dry_run);
        assert!(opts.skip_rc);
    }

    #[test]
    fn test_shell_type_debug() {
        // Test Debug trait implementation
        assert_eq!(format!("{:?}", ShellType::Bash), "Bash");
        assert_eq!(format!("{:?}", ShellType::Zsh), "Zsh");
        assert_eq!(format!("{:?}", ShellType::Fish), "Fish");
    }

    #[test]
    fn test_shell_type_clone() {
        let shell = ShellType::Bash;
        let cloned = shell.clone();
        assert_eq!(shell, cloned);
    }

    #[test]
    fn test_shell_type_copy() {
        let shell = ShellType::Zsh;
        let copied: ShellType = shell; // Copy happens here
        assert_eq!(shell, copied);
        // Both should still be usable (proves Copy, not just move)
        assert_eq!(shell.wrapper_filename(), "goto.zsh");
        assert_eq!(copied.wrapper_filename(), "goto.zsh");
    }

    #[test]
    fn test_shell_type_equality() {
        assert_eq!(ShellType::Bash, ShellType::Bash);
        assert_eq!(ShellType::Zsh, ShellType::Zsh);
        assert_eq!(ShellType::Fish, ShellType::Fish);
        assert_ne!(ShellType::Bash, ShellType::Zsh);
        assert_ne!(ShellType::Bash, ShellType::Fish);
        assert_ne!(ShellType::Zsh, ShellType::Fish);
    }

    #[test]
    fn test_wrapper_content_has_shebang_or_function() {
        // Bash and zsh wrappers should have shell function definitions
        let bash_content = ShellType::Bash.wrapper_content();
        let zsh_content = ShellType::Zsh.wrapper_content();
        let fish_content = ShellType::Fish.wrapper_content();

        // Bash/zsh typically define functions, fish uses function keyword
        assert!(bash_content.contains("function") || bash_content.contains("()"));
        assert!(zsh_content.contains("function") || zsh_content.contains("()"));
        assert!(fish_content.contains("function"));
    }

    #[test]
    fn test_detect_returns_error_for_unknown_shell() {
        // Save original SHELL env var
        let original = env::var("SHELL").ok();

        // Test with unknown shell
        env::set_var("SHELL", "/usr/bin/ksh");
        let result = ShellType::detect();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Could not auto-detect shell"));
        assert!(err.contains("ksh"));

        // Restore original
        match original {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
    }

    #[test]
    fn test_detect_bash() {
        let original = env::var("SHELL").ok();

        env::set_var("SHELL", "/bin/bash");
        let result = ShellType::detect();
        assert!(matches!(result, Ok(ShellType::Bash)));

        // Test with /usr/bin path
        env::set_var("SHELL", "/usr/bin/bash");
        let result = ShellType::detect();
        assert!(matches!(result, Ok(ShellType::Bash)));

        match original {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
    }

    #[test]
    fn test_detect_zsh() {
        let original = env::var("SHELL").ok();

        env::set_var("SHELL", "/bin/zsh");
        let result = ShellType::detect();
        assert!(matches!(result, Ok(ShellType::Zsh)));

        env::set_var("SHELL", "/usr/local/bin/zsh");
        let result = ShellType::detect();
        assert!(matches!(result, Ok(ShellType::Zsh)));

        match original {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
    }

    #[test]
    fn test_detect_fish() {
        let original = env::var("SHELL").ok();

        env::set_var("SHELL", "/usr/bin/fish");
        let result = ShellType::detect();
        assert!(matches!(result, Ok(ShellType::Fish)));

        match original {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
    }

    #[test]
    fn test_detect_empty_shell_env() {
        let original = env::var("SHELL").ok();

        env::set_var("SHELL", "");
        let result = ShellType::detect();
        assert!(result.is_err());

        match original {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
    }
}
