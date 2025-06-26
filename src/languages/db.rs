#![cfg(feature = "ssr")]
use duckdb::{params, Connection};
use once_cell::sync::OnceCell;
use std::sync::Mutex;

static INIT_TABLES_SQL: &str = "
CREATE TABLE IF NOT EXISTS words (
    word TEXT NOT NULL PRIMARY KEY,
    translation TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
);
";

static DATABASE: OnceCell<Mutex<Database>> = OnceCell::new();

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn get_instance(lang: &str) -> Result<&'static Mutex<Database>, anyhow::Error> {
        DATABASE.get_or_try_init(|| {
            let db = Database::load_or_init(&format!("db/{}.db", lang))?;
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

    // Idempotent.
    pub fn add_word(&self, word: &str) -> Result<(), anyhow::Error> {
        self.conn.execute("INSERT INTO words (word) VALUES (?) ON CONFLICT DO NOTHING", params![word])?;
        Ok(())
    }

}