//! Usage:
//! cargo run --bin=lang

use flashcard_app::languages::ai;
use flashcard_app::settings::Settings;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    Settings::load()?;
    ai::Agent::from_settings().populate_words_db().await?;
    Ok(())
}
