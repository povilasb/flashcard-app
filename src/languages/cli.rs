//! Usage:
//! cargo run --bin=lang --features=ssr

#![cfg(feature = "ssr")]

use dotenv::dotenv;
use flashcard_app::languages::ai;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load environment variables from .env file
    dotenv().ok();
    ai::Agent::new("spanish").populate_words_db().await.unwrap();
    Ok(())
}
