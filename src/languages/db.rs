#![cfg(feature = "ssr")]

use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Mutex;

use crate::languages::model::Word;

static INIT_TABLES_SQL: &str = "
CREATE TABLE IF NOT EXISTS words (
    word TEXT NOT NULL PRIMARY KEY,
    translation TEXT,
    created_at INTEGER DEFAULT (unixepoch('now'))
);
";

#[macro_export]
macro_rules! words_db {
    () => {
        crate::languages::db::Database::get_instance(
            &crate::settings::Settings::get().db_path,
            crate::settings::Settings::get().learning_language.as_str(),
        )
        .unwrap()
        .lock()
        .unwrap()
    };
}

static DATABASE: OnceCell<Mutex<Database>> = OnceCell::new();

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn get_instance(db_path: &str, lang: &str) -> rusqlite::Result<&'static Mutex<Database>> {
        DATABASE.get_or_try_init(|| {
            let db = Database::load_or_init(&format!("{}/{}.db", db_path, lang))?;
            Ok(Mutex::new(db))
        })
    }

    pub fn load_or_init(fname: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(fname)?;
        conn.execute_batch(INIT_TABLES_SQL)?;
        Ok(Self { conn })
    }

    // Idempotent.
    pub fn add_word(&self, word: &str, translation: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO words (word, translation) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            params![word, translation],
        )?;
        Ok(())
    }

    pub fn all_words(&self) -> rusqlite::Result<Vec<Word>> {
        let mut stmt = self.conn.prepare("SELECT word, translation, created_at FROM words")?;
        let words = stmt.query_map([], |row| {
            let ts: i64 = row.get(2)?;
            Ok(Word {
                word: row.get(0)?,
                translation: row.get(1)?,
                created_at: DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now),
            })
        })?;
        words.collect()
    }

    pub fn update_word_translation(&self, word: &str, translation: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE words SET translation = ?1 WHERE word = ?2",
            params![translation, word],
        )?;
        Ok(())
    }

    pub fn delete_word(&self, word: &str) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM words WHERE word = ?1", params![word])?;
        Ok(())
    }

    pub fn get_translation(&self, word: &str) -> rusqlite::Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT translation FROM words WHERE word = ?1",
                [word],
                |row| row.get(0),
            )
            .optional()
    }
}
