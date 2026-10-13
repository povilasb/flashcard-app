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
        let mut stmt = self
            .conn
            .prepare("SELECT word, translation, created_at FROM words")?;
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

    pub fn add_words(&mut self, words: &[String]) -> rusqlite::Result<()> {
        let tx = self.conn.transaction()?;
        for word in words {
            tx.execute(
                "INSERT INTO words (word, translation) VALUES (?1, '') ON CONFLICT DO NOTHING",
                [word],
            )?;
        }
        tx.commit()
    }

    pub fn update_word_translation(&self, word: &str, translation: &str) -> rusqlite::Result<()> {
        let changed = self.conn.execute(
            "UPDATE words SET translation = ?1 WHERE word = ?2",
            params![translation, word],
        )?;
        if changed == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    pub fn delete_word(&self, word: &str) -> rusqlite::Result<()> {
        if self
            .conn
            .execute("DELETE FROM words WHERE word = ?1", params![word])?
            == 0
        {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    }

    pub fn get_translation(&self, word: &str) -> rusqlite::Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT translation FROM words WHERE word = ?1",
                [word],
                |row| row.get::<_, Option<String>>(0),
            )
            .optional()
            .map(Option::flatten)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vocabulary_crud_preserves_duplicates_and_handles_null() {
        let mut db = Database::load_or_init(":memory:").unwrap();
        db.add_word("café", "coffee").unwrap();

        db.add_words(&["café".into(), "por favor".into()]).unwrap();

        assert_eq!(
            db.get_translation("café").unwrap().as_deref(),
            Some("coffee")
        );
        assert_eq!(db.all_words().unwrap().len(), 2);
        assert_eq!(db.get_translation("unknown").unwrap(), None);

        db.update_word_translation("por favor", "please").unwrap();
        db.conn
            .execute(
                "UPDATE words SET translation = NULL WHERE word = 'café'",
                [],
            )
            .unwrap();

        assert_eq!(db.get_translation("café").unwrap(), None);
        assert_eq!(
            db.get_translation("por favor").unwrap().as_deref(),
            Some("please")
        );

        db.delete_word("café").unwrap();

        assert!(matches!(
            db.delete_word("café"),
            Err(rusqlite::Error::QueryReturnedNoRows)
        ));
        assert!(matches!(
            db.update_word_translation("missing", "x"),
            Err(rusqlite::Error::QueryReturnedNoRows)
        ));
    }

    #[test]
    fn failed_import_rolls_back_all_insertions() {
        let mut db = Database::load_or_init(":memory:").unwrap();
        db.conn.execute_batch("CREATE TRIGGER fail_import BEFORE INSERT ON words WHEN NEW.word = 'bad' BEGIN SELECT RAISE(ABORT, 'fixture failure'); END;").unwrap();

        let result = db.add_words(&["good".into(), "bad".into()]);

        assert!(result.is_err());
        assert!(db.all_words().unwrap().is_empty());
    }
}
