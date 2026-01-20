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
    fn test_wrapper_content_not_empty() {
        assert!(!ShellType::Bash.wrapper_content().is_empty());
        assert!(!ShellType::Zsh.wrapper_content().is_empty());
        assert!(!ShellType::Fish.wrapper_content().is_empty());
    }

    #[test]
    fn test_wrapper_filename() {
        assert_eq!(ShellType::Bash.wrapper_filename(), "goto.bash");
        assert_eq!(ShellType::Zsh.wrapper_filename(), "goto.zsh");
        assert_eq!(ShellType::Fish.wrapper_filename(), "goto.fish");
    }
}
