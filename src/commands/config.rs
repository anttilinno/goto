//! Config command: show_config

use crate::config::Config;

/// Show the current configuration
pub fn show_config(config: &Config) {
    print!("{}", config.format_config());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_config_does_not_panic() {
        let config = Config::load().unwrap();
        // Just verify it doesn't panic
        show_config(&config);
    }
}
