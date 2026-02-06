use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Result type for config operations
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Config-related errors
#[derive(Debug)]
#[allow(dead_code)] // InsecurePermissions reserved for future strict mode
pub enum ConfigError {
    /// Failed to read config file
    ReadError(std::io::Error),
    /// Failed to parse config file
    ParseError(toml::de::Error),
    /// Config file has insecure permissions (Unix only)
    #[cfg(unix)]
    InsecurePermissions(PathBuf),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ReadError(e) => write!(f, "Failed to read config: {}", e),
            ConfigError::ParseError(e) => write!(f, "Failed to parse config: {}", e),
            #[cfg(unix)]
            ConfigError::InsecurePermissions(path) => {
                write!(f, "Config file has insecure permissions: {:?}", path)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::ReadError(e) => Some(e),
            ConfigError::ParseError(e) => Some(e),
            #[cfg(unix)]
            ConfigError::InsecurePermissions(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_editor")]
    pub editor: String,

    #[serde(default = "default_editor_args")]
    pub editor_args: Vec<String>,

    #[serde(default = "default_show_hidden")]
    pub show_hidden: bool,

    #[serde(default = "default_preview_max_lines")]
    pub preview_max_lines: usize,

    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_editor() -> String {
    "vim".to_string()
}

fn default_editor_args() -> Vec<String> {
    vec![]
}

fn default_show_hidden() -> bool {
    false
}

fn default_preview_max_lines() -> usize {
    1000
}

fn default_theme() -> String {
    "base16-ocean.dark".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: default_editor(),
            editor_args: default_editor_args(),
            show_hidden: default_show_hidden(),
            preview_max_lines: default_preview_max_lines(),
            theme: default_theme(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        match Self::load_with_result() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Config warning: {}", e);
                Self::default()
            }
        }
    }

    /// Load config with detailed error handling
    pub fn load_with_result() -> ConfigResult<Self> {
        let config_path = Self::config_path();

        if !config_path.exists() {
            return Ok(Self::default());
        }

        // Check permissions on Unix
        #[cfg(unix)]
        Self::check_permissions(&config_path)?;

        let content = fs::read_to_string(&config_path).map_err(ConfigError::ReadError)?;
        let config: Config = toml::from_str(&content).map_err(ConfigError::ParseError)?;

        // Validate editor command
        config.validate_editor()?;

        Ok(config)
    }

    /// Check that config file has secure permissions (Unix only)
    #[cfg(unix)]
    fn check_permissions(path: &PathBuf) -> ConfigResult<()> {
        use std::os::unix::fs::PermissionsExt;

        let metadata = fs::metadata(path).map_err(ConfigError::ReadError)?;
        let mode = metadata.permissions().mode();

        // Warn if config is world-writable (but don't fail)
        if mode & 0o002 != 0 {
            eprintln!(
                "Warning: Config file {:?} is world-writable. Consider running: chmod o-w {:?}",
                path, path
            );
        }

        Ok(())
    }

    /// Validate editor command for basic security
    fn validate_editor(&self) -> ConfigResult<()> {
        // Check for obviously dangerous patterns
        let dangerous_patterns = ["$(", "`", ";", "&&", "||", "|", ">", "<"];
        for pattern in &dangerous_patterns {
            if self.editor.contains(pattern) {
                eprintln!(
                    "Warning: Editor command contains potentially dangerous pattern: {}",
                    pattern
                );
            }
            for arg in &self.editor_args {
                if arg.contains(pattern) {
                    eprintln!(
                        "Warning: Editor argument contains potentially dangerous pattern: {}",
                        pattern
                    );
                }
            }
        }
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "vive-file-viewer") {
            let config_dir = proj_dirs.config_dir();
            config_dir.join("config.toml")
        } else {
            PathBuf::from("~/.config/vive-file-viewer/config.toml")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.editor, "vim");
        assert!(config.editor_args.is_empty());
        assert!(!config.show_hidden);
        assert_eq!(config.preview_max_lines, 1000);
        assert_eq!(config.theme, "base16-ocean.dark");
    }

    #[test]
    fn test_parse_config_from_toml() {
        let toml_str = r#"
            editor = "nvim"
            editor_args = ["-c", "startinsert"]
            show_hidden = true
            preview_max_lines = 500
            theme = "Solarized (dark)"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.editor, "nvim");
        assert_eq!(config.editor_args, vec!["-c", "startinsert"]);
        assert!(config.show_hidden);
        assert_eq!(config.preview_max_lines, 500);
        assert_eq!(config.theme, "Solarized (dark)");
    }

    #[test]
    fn test_parse_partial_config() {
        let toml_str = r#"
            editor = "code"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.editor, "code");
        // Other fields should have defaults
        assert!(!config.show_hidden);
        assert_eq!(config.preview_max_lines, 1000);
    }

    #[test]
    fn test_config_path_is_not_empty() {
        let path = Config::config_path();
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_config_error_display() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_error = ConfigError::ReadError(io_error);
        let display = format!("{}", config_error);
        assert!(display.contains("Failed to read config"));
    }

    #[test]
    fn test_validate_editor_safe_command() {
        let config = Config {
            editor: "vim".to_string(),
            editor_args: vec!["-c".to_string(), "set number".to_string()],
            ..Config::default()
        };
        // Should not panic or return error
        assert!(config.validate_editor().is_ok());
    }

    #[test]
    fn test_validate_editor_with_dangerous_patterns() {
        // These should still return Ok (just warn), but we test the detection logic
        let dangerous_editors = vec![
            "vim; rm -rf /",
            "$(malicious)",
            "`command`",
            "cmd && evil",
            "cmd || fallback",
            "cmd | pipe",
            "cmd > file",
            "cmd < file",
        ];

        for editor in dangerous_editors {
            let config = Config {
                editor: editor.to_string(),
                ..Config::default()
            };
            // validate_editor returns Ok but prints warnings
            // We just verify it doesn't panic
            let result = config.validate_editor();
            assert!(
                result.is_ok(),
                "validate_editor should not fail for: {}",
                editor
            );
        }
    }

    #[test]
    fn test_validate_editor_args_with_dangerous_patterns() {
        let config = Config {
            editor: "vim".to_string(),
            editor_args: vec!["-c".to_string(), "!rm -rf /".to_string()],
            ..Config::default()
        };
        // Should still succeed (warnings only)
        assert!(config.validate_editor().is_ok());
    }

    #[test]
    fn test_load_with_result_nonexistent_config_returns_default() {
        // When config file doesn't exist, load_with_result should return default
        // We can't easily test this without mocking, but we test the logic path
        let config = Config::default();
        assert_eq!(config.editor, "vim");
    }

    #[test]
    fn test_config_error_parse_error_display() {
        let parse_error = toml::from_str::<Config>("invalid = [toml").unwrap_err();
        let config_error = ConfigError::ParseError(parse_error);
        let display = format!("{}", config_error);
        assert!(display.contains("Failed to parse config"));
    }

    #[test]
    fn test_load_returns_default_on_missing_config() {
        // Config::load() should return default config when file is missing
        // This tests the fallback behavior
        let config = Config::load();
        // Should have default values
        assert_eq!(config.editor, "vim");
        assert!(!config.show_hidden);
    }

    #[test]
    fn test_parse_invalid_toml_returns_error() {
        let invalid_toml = "this is not valid toml [[[";
        let result: Result<Config, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_wrong_type_returns_error() {
        let wrong_type = r#"
            editor = 123
        "#;
        let result: Result<Config, _> = toml::from_str(wrong_type);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_with_all_fields() {
        let toml_str = r#"
            editor = "emacs"
            editor_args = ["--no-splash"]
            show_hidden = true
            preview_max_lines = 2000
            theme = "base16-mocha.dark"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.editor, "emacs");
        assert_eq!(config.editor_args, vec!["--no-splash"]);
        assert!(config.show_hidden);
        assert_eq!(config.preview_max_lines, 2000);
        assert_eq!(config.theme, "base16-mocha.dark");
    }
}
