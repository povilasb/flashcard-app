//! Usage:
//! cargo run --bin=cli --features=ssr

mod model;
mod db;

use std::env;
use std::error::Error;
use std::io;
use crate::db::Database;

static DB_DIR: &str = "flashcards";

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::load_or_init("flashcards.db")?;

    for card in db.cards_by_tag("rust".to_string(), 10)? {
        println!("Q: {} {}", card.last_reviewed, card.question);
    }

    return Ok(());

    let media_dir = env::current_dir()?.join(DB_DIR).join("media");

    while let Some(card) = db.next()? {
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
