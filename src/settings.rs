#[cfg(feature = "ssr")]
use anyhow::{Context, Result};
#[cfg(feature = "ssr")]
use config::Config;
#[cfg(feature = "ssr")]
use dirs;
#[cfg(feature = "ssr")]
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
#[cfg(feature = "ssr")]
use std::fs;
#[cfg(feature = "ssr")]
use std::path::Path;
use std::sync::OnceLock;

/// Supported languages.
#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy)]
pub enum Language {
    #[default]
    #[serde(rename = "spanish")]
    Spanish,
    #[serde(rename = "french")]
    French,
    #[serde(rename = "portuguese")]
    Portuguese,
    #[serde(rename = "german")]
    German,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Spanish => "spanish",
            Language::French => "french",
            Language::Portuguese => "portuguese",
            Language::German => "german",
        }
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub anthropic_api_key: String,
    pub learning_language: Language,
    pub db_path: String,
}

#[cfg(feature = "ssr")]
impl Default for Settings {
    fn default() -> Self {
        Self {
            anthropic_api_key: "".to_string(),
            learning_language: Language::Spanish,
            db_path: dirs::home_dir()
                .unwrap()
                .join("flashcards")
                .to_str()
                .unwrap()
                .to_string(),
        }
    }
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

impl Settings {
    /// Get a reference to a global Settings instance.
    ///
    /// # Panics
    ///
    /// Panics if the Settings instance is not loaded. Make sure to call Settings::load() from main.rs.
    pub fn get() -> &'static Self {
        SETTINGS.get().unwrap_or_else(|| {
            panic!("Settings not loaded. Make sure to call Settings::load() from main.rs.");
        })
    }

    /// Call this from main.rs.
    #[cfg(feature = "ssr")]
    pub fn load() -> Result<Self, config::ConfigError> {
        let cfg_dir = dirs::config_dir()
            .ok_or(config::ConfigError::NotFound("config_dir".to_string()))?
            .join("flashcard-app");
        std::fs::create_dir_all(&cfg_dir)
            .with_context(|| format!("Failed to create config directory {:?}", cfg_dir))
            .map_err(|e| config::ConfigError::Foreign(e.into()))?;
        info!("Using settings dir {:?}", cfg_dir);

        let cfg = Config::builder()
            // Load from environment variables (overrides file)
            .add_source(
                config::Environment::with_prefix("FLASHCARDS")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            );

        let settings_path = if Path::new("settings.toml").exists() {
            info!("Loading settings from ./settings.toml");
            Path::new("settings.toml").to_path_buf()
        } else {
            let settings_path = cfg_dir.join("settings.toml");
            if settings_path.exists() {
                info!("Loading settings from {:?}", settings_path);
            } else {
                info!(
                    "Creating settings at {:?} - make sure to set AI API keys.",
                    settings_path
                );
                Settings::default()
                    .save(&settings_path)
                    .map_err(|e| config::ConfigError::Foreign(e.into()))?;
            }
            settings_path
        };

        let cfg = cfg.add_source(config::File::with_name(settings_path.to_str().unwrap()));

        let settings = cfg.build()?.try_deserialize::<Settings>()?;
        SETTINGS.set(settings.clone()).unwrap();
        Ok(settings)
    }

    /// Save settings to a TOML file at the specified path.
    #[cfg(feature = "ssr")]
    pub fn save(&self, settings_path: &Path) -> Result<()> {
        let toml_string =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize settings to TOML")?;

        fs::write(settings_path, toml_string)
            .with_context(|| format!("Failed to write settings to {:?}", settings_path))?;

        Ok(())
    }
}
