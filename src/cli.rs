//! Usage:
//! cargo run --bin=cli --features=ssr

mod db;
mod model;

use crate::db::Database;
use std::env;
use std::error::Error;
use std::io;

static DB_DIR: &str = "db";

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::load_or_init("db/flashcards.db")?;

    let media_dir = env::current_dir()?.join(DB_DIR).join("media");

    for card in db.cards_to_review()? {
        println!("Q: {}", card.question);
        if let Some(img) = card.img {
            println!("   file://{}", media_dir.join(img).to_str().unwrap());
        }
        println!("   #{}", card.tags.join(", "));
        println!("Press enter to reveal the answer");
        readln();

        println!("A: {}", card.answer);
        println!("OK? (y/n): ");
        let inpt = readln();
        match inpt.as_str() {
            "y" => {
                db.ok(card.id)?;
            }
            "n" => {
                db.fail(card.id)?;
            }
            _ => {
                println!("Invalid input");
            }
        }
    }

    Ok(())
}

fn readln() -> String {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim().to_string()
}
