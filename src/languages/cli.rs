//! Usage:
//! cargo run --bin=lang --features=ssr
 
#![cfg(feature = "ssr")]

use dotenv::dotenv;
use flashcard_app::languages::ai::populate_words_db;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load environment variables from .env file
    dotenv().ok();
    populate_words_db("spanish").await?;
    Ok(())
}