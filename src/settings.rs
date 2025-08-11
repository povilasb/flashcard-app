use config::Config;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
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

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Settings {
    pub anthropic_api_key: String,
    pub learning_language: Language,
    pub db_path: String,
}

static SETTINGS: OnceLock<Settings> = OnceLock::new();

impl Settings {
    pub fn get() -> &'static Self {
        SETTINGS.get().unwrap_or_else(|| {
            panic!("Settings not loaded. Make sure to call Settings::load() from main.rs.");
        })
    }

    /// Call this from main.rs.
    pub fn load() -> Result<Self, config::ConfigError> {
        let cfg = Config::builder()
            // Load from config file
            .add_source(config::File::with_name("settings"))
            // Load from environment variables (overrides file)
            .add_source(
                config::Environment::with_prefix("FLASHCARDS")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            )
            .build()?;
        let settings = cfg.try_deserialize::<Settings>()?;
        SETTINGS.set(settings.clone()).unwrap();
        Ok(settings)
    }
}
