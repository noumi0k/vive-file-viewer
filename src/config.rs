use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
        let config_path = Self::config_path();

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => {
                        eprintln!("Failed to parse config: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read config: {}", e);
                }
            }
        }

        Self::default()
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
