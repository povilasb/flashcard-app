mod flashcard;

use std::error::Error;
use std::io;

use flashcard::Database;

const DB_DIR: &str = "flashcards";

fn main() -> Result<(), Box<dyn Error>> {
    let mut db = Database::load(DB_DIR)?;

    for mut card in db.review() {
        println!("Q: {}", card.question());
        println!("Press enter to reveal the answer");
        readln();

        println!("A: {}", card.answer());
        println!("OK? (y/n): ");
        let inpt = readln();
        match inpt.as_str() {
            "y" => {
                card.ok();
            }
            "n" => {
                card.fail();
            }
            _ => {
                println!("Invalid input");
            }
        }
    }

    db.save()?;

    Ok(())
}

fn readln() -> String {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
    buffer.trim().to_string()
}
