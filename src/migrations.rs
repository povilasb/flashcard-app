use chrono::Utc;
use duckdb::{Connection, Result};
use std::fs;
use std::path::Path;

pub struct Migration {
    pub version: i32,
    pub name: String,
    pub sql: String,
}

pub fn init_migrations_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TIMESTAMP NOT NULL
        )
    ",
    )
}

pub fn get_applied_migrations(conn: &Connection) -> Result<Vec<i32>> {
    let mut stmt = conn.prepare("SELECT version FROM schema_migrations ORDER BY version")?;
    let versions = stmt
        .query_map([], |row| Ok(row.get(0)?))?
        .collect::<Result<Vec<i32>, _>>()?;
    Ok(versions)
}

pub fn apply_migration(conn: &Connection, migration: &Migration) -> Result<()> {
    conn.execute_batch(&migration.sql)?;

    conn.execute(
        "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?, ?, ?)",
        duckdb::params![migration.version, &migration.name, Utc::now()],
    )?;

    Ok(())
}

pub fn load_migrations(migrations_dir: &Path) -> Result<Vec<Migration>> {
    let mut migrations = Vec::new();

    for entry in fs::read_dir(migrations_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("sql") {
            let filename = path.file_stem().unwrap().to_string_lossy();
            let parts: Vec<&str> = filename.split('_').collect();

            if parts.len() >= 2 {
                if let Ok(version) = parts[0].parse::<i32>() {
                    let name = parts[1..].join("_");
                    let sql = fs::read_to_string(&path)?;

                    migrations.push(Migration { version, name, sql });
                }
            }
        }
    }

    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}

pub fn run_migrations(conn: &Connection, migrations_dir: &Path) -> Result<()> {
    init_migrations_table(conn)?;
    let applied = get_applied_migrations(conn)?;
    let migrations = load_migrations(migrations_dir)?;

    for migration in migrations {
        if !applied.contains(&migration.version) {
            println!(
                "Applying migration {}: {}",
                migration.version, migration.name
            );
            apply_migration(conn, &migration)?;
        }
    }

    Ok(())
}
