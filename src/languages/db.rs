#![cfg(feature = "ssr")]

use duckdb::{params, types::Value, Connection, Error as DuckdbError, Result};
use once_cell::sync::OnceCell;
use std::sync::Mutex;

use crate::db::from_duckdb_timestamp;
use crate::languages::model::Word;

static INIT_TABLES_SQL: &str = "
CREATE TABLE IF NOT EXISTS words (
    word TEXT NOT NULL PRIMARY KEY,
    translation TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
);
";

#[macro_export]
macro_rules! words_db {
    ($lang:expr) => {
        crate::languages::db::Database::get_instance($lang)
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
    pub fn get_instance(lang: &str) -> Result<&'static Mutex<Database>, DuckdbError> {
        DATABASE.get_or_try_init(|| {
            let db = Database::load_or_init(&format!("db/{}.db", lang))?;
            Ok(Mutex::new(db))
        })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, DuckdbError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(INIT_TABLES_SQL)?;
        Ok(Self { conn })
    }

    // Load existing db or create a new one if it doesn't exist.
    pub fn load_or_init(fname: &str) -> Result<Self, DuckdbError> {
        let conn = Connection::open(fname)?;
        conn.execute_batch(INIT_TABLES_SQL)?;
        Ok(Self { conn })
    }

    // Idempotent.
    pub fn add_word(&self, word: &str, translation: &str) -> Result<(), DuckdbError> {
        self.conn.execute(
            "INSERT INTO words (word, translation) VALUES (?, ?) ON CONFLICT DO NOTHING",
            params![word, translation],
        )?;
        Ok(())
    }

    pub fn all_words(&self) -> Result<Vec<Word>, DuckdbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT word, translation, created_at FROM words")?;
        let words = stmt.query_map(params![], |row| {
            Ok(Word {
                word: row.get(0)?,
                translation: row.get(1)?,
                created_at: from_duckdb_timestamp(row.get::<_, Value>(2)?),
            })
        })?;
        Ok(words.collect::<Result<Vec<Word>, _>>()?)
    }

    pub fn update_word_translation(
        &self,
        word: &str,
        translation: &str,
    ) -> Result<(), DuckdbError> {
        self.conn.execute(
            "UPDATE words SET translation = ? WHERE word = ?",
            params![translation, word],
        )?;
        Ok(())
    }

    pub fn delete_word(&self, word: &str) -> Result<(), DuckdbError> {
        self.conn
            .execute("DELETE FROM words WHERE word = ?", params![word])?;
        Ok(())
    }

    pub fn get_translation(&self, word: &str) -> Result<Option<String>, DuckdbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT translation FROM words WHERE word = ?")?;
        let mut rows = stmt.query_map(params![word], |row| Ok(row.get::<_, String>(0)?))?;

        if let Some(Ok(translation)) = rows.next() {
            Ok(Some(translation))
        } else {
            Ok(None)
        }
    }
}
