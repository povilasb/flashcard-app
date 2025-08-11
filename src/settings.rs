use config::Config;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Settings {
    pub anthropic_api_key: String,
    pub learning_language: String,
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
