//! duckdb-based database for flashcards.

#![cfg(feature = "ssr")]

use crate::settings::Settings;
use anyhow::Result;
use chrono::{DateTime, Utc};
use duckdb::types::Value;
use duckdb::{params, Connection, Error as DuckdbError};
use once_cell::sync::OnceCell;
use std::error::Error;
use std::sync::Mutex;

use crate::model::{Flashcard, ReviewHistory};

/// NOTES:
/// * duckdb-rs doesn't support arrays, so tags are stored in a separate table.
///   * https://github.com/duckdb/duckdb-rs/issues/338
static INIT_TABLES_SQL: &str = "
    CREATE SEQUENCE seq_flashcards;
    CREATE TABLE IF NOT EXISTS flashcards (
        id INTEGER PRIMARY KEY DEFAULT NEXTVAL('seq_flashcards'),
        question TEXT,
        answer TEXT,
        examples TEXT,
        source TEXT,
        img TEXT,
        last_reviewed TIMESTAMP,
        review_after_secs INTEGER,
        question_img TEXT,
    );

    CREATE TABLE IF NOT EXISTS flashcard_tags (
        flashcard_id INTEGER,
        tag TEXT,
        PRIMARY KEY (flashcard_id, tag),
        FOREIGN KEY (flashcard_id) REFERENCES flashcards(id),
    );

    CREATE TABLE IF NOT EXISTS review_history (
        flashcard_id INTEGER,
        review_date TIMESTAMP,
        remembered BOOLEAN,
        PRIMARY KEY (flashcard_id, review_date),
        FOREIGN KEY (flashcard_id) REFERENCES flashcards(id),
    );
";

static DATABASE: OnceCell<Mutex<Database>> = OnceCell::new();

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn get_instance() -> Result<&'static Mutex<Database>, anyhow::Error> {
        DATABASE.get_or_try_init(|| {
            let db = Database::load_or_init(&format!("{}/flashcards.db", Settings::get().db_path))?;
            Ok(Mutex::new(db))
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, anyhow::Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(INIT_TABLES_SQL)?;
        Ok(Self { conn })
    }

    // Load existing db or create a new one if it doesn't exist.
    pub fn load_or_init(fname: &str) -> Result<Self, anyhow::Error> {
        let conn = Connection::open(fname)?;
        conn.execute_batch(INIT_TABLES_SQL)?;
        Ok(Self { conn })
    }

    pub fn add_card(&self, card: &Flashcard) -> Result<(), anyhow::Error> {
        self.conn.execute("BEGIN TRANSACTION", params![])?;

        let mut stmt = self.conn.prepare(
            "INSERT INTO flashcards (question, answer, examples, source, img, question_img, last_reviewed, review_after_secs) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id"
        )?;
        let flashcard_id: i64 = stmt.query_row(
            params![
                card.question,
                card.answer,
                card.examples,
                card.source,
                card.img,
                card.question_img,
                card.last_reviewed.to_rfc3339(),
                card.review_after_secs,
            ],
            |row| row.get(0),
        )?;

        for tag in card.tags.iter() {
            self.conn.execute(
                "INSERT INTO flashcard_tags (flashcard_id, tag) VALUES (?, ?)",
                params![flashcard_id, tag],
            )?;
        }

        self.conn.execute("COMMIT", params![])?;
        Ok(())
    }

    pub fn all_cards(&self, tag: Option<String>) -> Result<Vec<Flashcard>, DuckdbError> {
        let mut query = "SELECT f.*, group_concat(ft.tag) from flashcards f 
            join flashcard_tags ft on f.id = ft.flashcard_id"
            .to_string();
        if let Some(tag) = tag {
            query += &format!(" WHERE ft.tag = '{}'", tag);
        }
        query += " GROUP BY f.id, f.question, f.answer, f.examples, f.source, f.img, f.last_reviewed, f.review_after_secs, f.question_img";

        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map([], |row| self.flashcard_from_row(row))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn cards_to_review(&self) -> Result<Vec<Flashcard>, anyhow::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT f.*, group_concat(ft.tag) from flashcards f 
            join flashcard_tags ft on f.id = ft.flashcard_id 
            WHERE last_reviewed + INTERVAL(review_after_secs) SECOND < CURRENT_TIMESTAMP
            GROUP BY f.id, f.question, f.answer, f.examples, f.source, f.img, f.question_img, f.last_reviewed, f.review_after_secs")?;
        let rows = stmt.query_map([], |row| self.flashcard_from_row(row))?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn ok(&self, card_id: i64) -> Result<(), Box<dyn Error>> {
        self.conn.execute("BEGIN TRANSACTION", params![])?;
        self.conn.execute("UPDATE flashcards SET last_reviewed = CURRENT_TIMESTAMP, review_after_secs = review_after_secs * 2 WHERE id = ?", params![card_id])?;
        self.conn.execute("INSERT INTO review_history (flashcard_id, review_date, remembered) VALUES (?, CURRENT_TIMESTAMP, TRUE)", params![card_id])?;
        self.conn.execute("COMMIT", params![])?;
        Ok(())
    }

    pub fn fail(&self, card_id: i64) -> Result<(), Box<dyn Error>> {
        self.conn.execute("BEGIN TRANSACTION", params![])?;
        // Don't prompt to review immediately.
        self.conn.execute("UPDATE flashcards SET last_reviewed = CURRENT_TIMESTAMP, review_after_secs = 3600 WHERE id = ?", params![card_id])?;
        self.conn.execute("INSERT INTO review_history (flashcard_id, review_date, remembered) VALUES (?, CURRENT_TIMESTAMP, FALSE)", params![card_id])?;
        self.conn.execute("COMMIT", params![])?;
        Ok(())
    }

    pub fn get_card(&self, id: i64) -> Result<Flashcard, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.*, group_concat(ft.tag) from flashcards f 
            join flashcard_tags ft on f.id = ft.flashcard_id 
            WHERE f.id = ?
            GROUP BY f.id, f.question, f.answer, f.examples, f.source, f.img, f.question_img, f.last_reviewed, f.review_after_secs"
        )?;
        let card = stmt.query_row([id], |row| self.flashcard_from_row(row))?;
        Ok(card)
    }

    pub fn update_card(&self, card: &Flashcard) -> Result<(), Box<dyn Error>> {
        // NOTE: transactions don't work: seems like duckdb doesn't see flashcard_tags being removed when trying
        // to insert new tags:
        //     Some("Constraint Error: Duplicate key \"flashcard_id: 1, tag: tag1\" violates primary key constraint.
        //self.conn.execute("BEGIN TRANSACTION", params![])?;

        // Update the flashcard
        self.conn.execute(
            "UPDATE flashcards SET question = ?, answer = ?, examples = ?, source = ?, img = ?, question_img = ? WHERE id = ?",
            params![
                card.question,
                card.answer,
                card.examples,
                card.source,
                card.img,
                card.question_img,
                card.id,
            ]
        )?;

        // Delete existing tags
        self.conn.execute(
            "DELETE FROM flashcard_tags WHERE flashcard_id = ?",
            params![card.id],
        )?;

        // Insert new tags
        for tag in card.tags.iter().as_ref() {
            self.conn.execute(
                "INSERT INTO flashcard_tags (flashcard_id, tag) VALUES (?, ?)",
                params![card.id, tag],
            )?;
        }

        //self.conn.execute("COMMIT", params![])?;
        Ok(())
    }

    pub fn review_history(&self) -> Result<Vec<ReviewHistory>, anyhow::Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM review_history")?;
        let rows = stmt.query_map([], |row| {
            Ok(ReviewHistory {
                flashcard_id: row.get::<_, i64>(0)?,
                review_date: from_duckdb_timestamp(row.get::<_, Value>(1)?),
                remembered: row.get::<_, bool>(2)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Helper function to construct a Flashcard from a database row
    fn flashcard_from_row(&self, row: &duckdb::Row) -> Result<Flashcard, duckdb::Error> {
        Ok(Flashcard {
            id: row.get::<_, i64>(0)?,
            question: row.get(1)?,
            answer: row.get(2)?,
            examples: row.get(3)?,
            source: row.get(4)?,
            img: row.get(5)?,
            last_reviewed: from_duckdb_timestamp(row.get::<_, Value>(6)?),
            review_after_secs: row.get(7)?,
            question_img: row.get(8)?,
            tags: row
                .get::<_, String>(9)?
                .split(",")
                .map(|s| s.to_string())
                .collect(),
        })
    }
}

pub fn from_duckdb_timestamp(t: Value) -> DateTime<Utc> {
    match t {
        Value::Timestamp(time_unit, value) => {
            DateTime::from_timestamp_micros(time_unit.to_micros(value))
                .unwrap()
                .with_timezone(&Utc)
        }
        _ => panic!("expected timestamp, got {:?}", t),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Flashcard;

    #[test]
    fn test_update_card_works_when_nothing_changed() {
        let db = Database::in_memory().unwrap();
        let mut card = Flashcard::new("question1".to_string(), "answer1".to_string());
        card.tags = vec!["tag1".to_string()];
        db.add_card(&card).unwrap();

        card.id = 1;
        db.update_card(&card).unwrap();

        let card = db.get_card(1).unwrap();
        assert_eq!(card.tags, vec!["tag1".to_string()]);
    }

    #[test]
    fn test_ok_appends_to_review_history() {
        let db = Database::in_memory().unwrap();
        let card = Flashcard::new("question1".to_string(), "answer1".to_string());
        db.add_card(&card).unwrap();

        db.ok(1).unwrap();

        let review_history = db.review_history().unwrap();
        assert_eq!(review_history.len(), 1);
        assert_eq!(review_history[0].flashcard_id, 1);
        assert_eq!(
            review_history[0].review_date.format("%Y-%m-%d").to_string(),
            Utc::now().format("%Y-%m-%d").to_string()
        );
        assert_eq!(review_history[0].remembered, true);
    }
}
