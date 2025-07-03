mod model;

use anyhow::Context;
use glob::glob;
use model::Flashcard;
use std::error::Error;
use std::fs;
use toml;

fn main() -> Result<(), Box<dyn Error>> {
    for fname in glob("../flashcards/**/*.toml")? {
        let fname = fname?;
        let contents = fs::read_to_string(&fname)?;
        let mut card: Flashcard = toml::from_str(&contents)?;

        card.tags = vec![fname
            .to_string_lossy()
            .split('/')
            .rev()
            .nth(1)
            .unwrap_or_else(|| "unknown")
            .to_string()];

        println!("{} {} {:?}", card.id, card.question, card.tags);

        let toml = toml::to_string(&card).context("Failed to serialize card")?;
        fs::write(&fname, toml).context("Failed to write card to disk")?;
    }
    Ok(())
}
