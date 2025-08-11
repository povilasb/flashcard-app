//! Usage:
//! cargo run --bin=lang --features=ssr

#![cfg(feature = "ssr")]

use flashcard_app::languages::ai;
use flashcard_app::settings::Language;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    ai::Agent::new(Language::Spanish, "")
        .populate_words_db()
        .await
        .unwrap();
    Ok(())
}
