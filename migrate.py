#!/usr/bin/env python3
# /// script
# dependencies = [
#   "duckdb",
#   "typer",
# ]
# ///
"""Migrate flashcard and language data from DuckDB to SQLite.

Usage:
    uv run migrate.py flashcards
    uv run migrate.py language spanish
    uv run migrate.py language german
"""

import sqlite3
import duckdb
import typer
from pathlib import Path

app = typer.Typer()


def _to_unix(ts) -> int:
    if ts is None:
        return 0
    return int(ts.timestamp())


def _migrate_flashcards():
    duck = duckdb.connect("db.bk/flashcards.db", read_only=True)
    sq = sqlite3.connect("db/flashcards.db")

    sq.executescript("""
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
        PRIMARY KEY (flashcard_id, tag)
    );
    CREATE TABLE IF NOT EXISTS review_history (
        flashcard_id INTEGER,
        review_date INTEGER,
        remembered INTEGER,
        PRIMARY KEY (flashcard_id, review_date)
    );
    """)

    rows = duck.execute(
        "SELECT id, question, answer, examples, source, img, last_reviewed, review_after_secs, question_img FROM flashcards"
    ).fetchall()
    sq.executemany(
        "INSERT INTO flashcards VALUES (?,?,?,?,?,?,?,?,?)",
        [(r[0], r[1], r[2], r[3], r[4], r[5], _to_unix(r[6]), r[7], r[8]) for r in rows],
    )
    print(f"  {len(rows)} flashcards")

    rows = duck.execute("SELECT flashcard_id, tag FROM flashcard_tags").fetchall()
    sq.executemany("INSERT INTO flashcard_tags VALUES (?,?)", rows)
    print(f"  {len(rows)} tags")

    rows = duck.execute("SELECT flashcard_id, review_date, remembered FROM review_history").fetchall()
    sq.executemany(
        "INSERT INTO review_history VALUES (?,?,?)",
        [(r[0], _to_unix(r[1]), int(r[2])) for r in rows],
    )
    print(f"  {len(rows)} review history entries")

    sq.commit()

    duck_count = duck.execute("SELECT count(*) FROM flashcards").fetchone()[0]
    sq_count = sq.execute("SELECT count(*) FROM flashcards").fetchone()[0]
    assert duck_count == sq_count, f"Row count mismatch: DuckDB={duck_count}, SQLite={sq_count}"

    duck.close()
    sq.close()


def _migrate_language(lang: str):
    duck = duckdb.connect(f"db.bk/{lang}.db", read_only=True)
    sq = sqlite3.connect(f"db/{lang}.db")

    sq.executescript("""
    CREATE TABLE IF NOT EXISTS words (
        word TEXT NOT NULL PRIMARY KEY,
        translation TEXT,
        created_at INTEGER DEFAULT (unixepoch('now'))
    );
    """)

    rows = duck.execute("SELECT word, translation, created_at FROM words").fetchall()
    sq.executemany(
        "INSERT INTO words VALUES (?,?,?)",
        [(r[0], r[1], _to_unix(r[2])) for r in rows],
    )
    sq.commit()
    print(f"  {len(rows)} words")

    duck.close()
    sq.close()


@app.command()
def flashcards():
    """Migrate flashcards.db from DuckDB to SQLite."""
    Path("db").mkdir(exist_ok=True)
    print("Migrating flashcards...")
    _migrate_flashcards()
    print("Done.")


@app.command()
def language(lang: str = typer.Argument(..., help="Language name, e.g. spanish, german")):
    """Migrate a language words DB from DuckDB to SQLite."""
    Path("db").mkdir(exist_ok=True)
    print(f"Migrating {lang}...")
    _migrate_language(lang)
    print("Done.")


if __name__ == "__main__":
    app()
