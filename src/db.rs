//! SQLite-based database for flashcards.

#![cfg(feature = "ssr")]

use crate::model::{Flashcard, ReviewHistory};
use crate::settings::Settings;
use anyhow::Result;
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use rusqlite::{params, Connection};
use std::error::Error;
use std::sync::Mutex;

static INIT_TABLES_SQL: &str = "
    CREATE TABLE IF NOT EXISTS flashcards (
        id INTEGER PRIMARY KEY,
        question TEXT,
        answer TEXT,
        examples TEXT,
        source TEXT,
        img TEXT,
        last_reviewed INTEGER,
        review_after_secs INTEGER,
        question_img TEXT
    );

    CREATE TABLE IF NOT EXISTS flashcard_tags (
        flashcard_id INTEGER,
        tag TEXT,
        PRIMARY KEY (flashcard_id, tag),
        FOREIGN KEY (flashcard_id) REFERENCES flashcards(id)
    );

    CREATE TABLE IF NOT EXISTS review_history (
        flashcard_id INTEGER,
        review_date INTEGER,
        remembered BOOLEAN,
        PRIMARY KEY (flashcard_id, review_date),
        FOREIGN KEY (flashcard_id) REFERENCES flashcards(id)
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

    pub fn load_or_init(fname: &str) -> Result<Self, anyhow::Error> {
        let conn = Connection::open(fname)?;
        conn.execute_batch(INIT_TABLES_SQL)?;
        Ok(Self { conn })
    }

    pub fn add_card(&mut self, card: &Flashcard) -> Result<(), anyhow::Error> {
        let tx = self.conn.transaction()?;
        let flashcard_id: i64 = tx.query_row(
            "INSERT INTO flashcards (question, answer, examples, source, img, question_img, last_reviewed, review_after_secs)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) RETURNING id",
            params![
                card.question,
                card.answer,
                card.examples,
                card.source,
                card.img,
                card.question_img,
                card.last_reviewed.timestamp(),
                card.review_after_secs,
            ],
            |row| row.get(0),
        )?;
        for tag in &card.tags {
            tx.execute(
                "INSERT INTO flashcard_tags (flashcard_id, tag) VALUES (?1, ?2)",
                params![flashcard_id, tag],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn all_cards(&self, tag: Option<String>) -> rusqlite::Result<Vec<Flashcard>> {
        let group_by = "GROUP BY f.id, f.question, f.answer, f.examples, f.source, f.img, f.last_reviewed, f.review_after_secs, f.question_img";
        let from = "SELECT f.*, group_concat(ft.tag) FROM flashcards f
            LEFT JOIN flashcard_tags ft ON f.id = ft.flashcard_id";

        let sql = match &tag {
            Some(_) => format!("{from} WHERE ft.tag = ?1 {group_by}"),
            None => format!("{from} {group_by}"),
        };
        let mut stmt = self.conn.prepare(&sql)?;
        match tag {
            Some(t) => stmt.query_map([t], flashcard_from_row)?.collect(),
            None => stmt.query_map([], flashcard_from_row)?.collect(),
        }
    }

    pub fn cards_to_review(&self) -> Result<Vec<Flashcard>, anyhow::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT f.*, group_concat(ft.tag) FROM flashcards f
            LEFT JOIN flashcard_tags ft ON f.id = ft.flashcard_id
            WHERE f.last_reviewed + f.review_after_secs < unixepoch('now')
            GROUP BY f.id, f.question, f.answer, f.examples, f.source, f.img, f.question_img, f.last_reviewed, f.review_after_secs",
        )?;
        let rows = stmt.query_map([], flashcard_from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn ok(&mut self, card_id: i64) -> Result<(), Box<dyn Error>> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE flashcards SET last_reviewed = unixepoch('now'), review_after_secs = review_after_secs * 2 WHERE id = ?1",
            params![card_id],
        )?;
        tx.execute(
            "INSERT INTO review_history (flashcard_id, review_date, remembered) VALUES (?1, unixepoch('now'), TRUE)",
            params![card_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn fail(&mut self, card_id: i64) -> Result<(), Box<dyn Error>> {
        let tx = self.conn.transaction()?;
        // Don't prompt to review immediately.
        // Review no earlier than after 6 hours.
        tx.execute(
            "UPDATE flashcards SET last_reviewed = unixepoch('now'), review_after_secs = 21600 WHERE id = ?1",
            params![card_id],
        )?;
        tx.execute(
            "INSERT INTO review_history (flashcard_id, review_date, remembered) VALUES (?1, unixepoch('now'), FALSE)",
            params![card_id],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_card(&self, id: i64) -> Result<Flashcard, Box<dyn Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT f.*, group_concat(ft.tag) FROM flashcards f
            LEFT JOIN flashcard_tags ft ON f.id = ft.flashcard_id
            WHERE f.id = ?1
            GROUP BY f.id, f.question, f.answer, f.examples, f.source, f.img, f.question_img, f.last_reviewed, f.review_after_secs",
        )?;
        Ok(stmt.query_row([id], flashcard_from_row)?)
    }

    pub fn update_card(&mut self, card: &Flashcard) -> Result<(), Box<dyn Error>> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE flashcards SET question = ?1, answer = ?2, examples = ?3, source = ?4, img = ?5, question_img = ?6 WHERE id = ?7",
            params![
                card.question,
                card.answer,
                card.examples,
                card.source,
                card.img,
                card.question_img,
                card.id,
            ],
        )?;
        tx.execute(
            "DELETE FROM flashcard_tags WHERE flashcard_id = ?1",
            params![card.id],
        )?;
        for tag in &card.tags {
            tx.execute(
                "INSERT INTO flashcard_tags (flashcard_id, tag) VALUES (?1, ?2)",
                params![card.id, tag],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn delete_card(&mut self, id: i64) -> Result<(), Box<dyn Error>> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM flashcard_tags WHERE flashcard_id = ?1", params![id])?;
        tx.execute("DELETE FROM flashcards WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn review_history(&self) -> Result<Vec<ReviewHistory>, anyhow::Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM review_history")?;
        let rows = stmt.query_map([], |row| {
            let ts: i64 = row.get(1)?;
            Ok(ReviewHistory {
                flashcard_id: row.get(0)?,
                review_date: DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now),
                remembered: row.get(2)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn avg_reviews_per_month(&self) -> Result<f64, rusqlite::Error> {
        let query = "
        SELECT avg(reviews) FROM (
            SELECT
                flashcard_id,
                strftime('%Y-%m', review_date, 'unixepoch') as year_month,
                count(*) as reviews
            FROM review_history
            GROUP BY flashcard_id, year_month
        )";
        let mut stmt = self.conn.prepare(query)?;
        stmt.query_row([], |row| row.get(0))
    }
}

fn flashcard_from_row(row: &rusqlite::Row) -> rusqlite::Result<Flashcard> {
    let ts: i64 = row.get(6)?;
    Ok(Flashcard {
        id: row.get(0)?,
        question: row.get(1)?,
        answer: row.get(2)?,
        examples: row.get(3)?,
        source: row.get(4)?,
        img: row.get(5)?,
        last_reviewed: DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now),
        review_after_secs: row.get(7)?,
        question_img: row.get(8)?,
        tags: row
            .get::<_, Option<String>>(9)?
            .map(|s| s.split(',').map(str::to_string).collect())
            .unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_card_works_when_nothing_changed() {
        let mut db = Database::in_memory().unwrap();
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
        let mut db = Database::in_memory().unwrap();
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
