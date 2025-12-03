use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// OpenAI model to use (default: gpt-4)
    #[serde(default = "default_model")]
    pub model: String,

    /// Maximum tokens for responses
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Multi-turn depth for tool calls
    #[serde(default = "default_multi_turn_depth")]
    pub multi_turn_depth: usize,

    /// Enable direct command execution
    #[serde(default = "default_direct_commands")]
    pub enable_direct_commands: bool,

    /// Show intent classification in output
    #[serde(default = "default_show_intent")]
    pub show_intent: bool,
}

fn default_model() -> String {
    "gpt-4".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_multi_turn_depth() -> usize {
    10
}

fn default_direct_commands() -> bool {
    true
}

fn default_show_intent() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: default_model(),
            max_tokens: default_max_tokens(),
            multi_turn_depth: default_multi_turn_depth(),
            enable_direct_commands: default_direct_commands(),
            show_intent: default_show_intent(),
        }
    }
}

impl Config {
    /// Get the config directory path (~/.ada)
    pub fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".ada"))
    }

    /// Get the config file path (~/.ada/config)
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config"))
    }

    /// Load configuration from ~/.ada/config or create default if not exists
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        let config_file = Self::config_file_path()?;

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
            eprintln!("Created config directory: {}", config_dir.display());
        }

        // If config file exists, read it
        if config_file.exists() {
            let contents = fs::read_to_string(&config_file)
                .context("Failed to read config file")?;

            let config: Config = toml::from_str(&contents)
                .context("Failed to parse config file")?;

            eprintln!("Loaded config from: {}", config_file.display());
            Ok(config)
        } else {
            // Create default config file
            let default_config = Config::default();
            default_config.save()?;
            eprintln!("Created default config at: {}", config_file.display());
            Ok(default_config)
        }
    }

    /// Save configuration to ~/.ada/config
    pub fn save(&self) -> Result<()> {
        let config_file = Self::config_file_path()?;
        let toml_string = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_file, toml_string)
            .context("Failed to write config file")?;

        Ok(())
    }
}
