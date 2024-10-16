use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fs;
use std::path::PathBuf;
use ulid::Ulid;

#[derive(Debug)]
pub struct Database {
    sorted_cards: Vec<CardFromFileSys>,
}

impl Database {
    pub fn load(dir: &str) -> Result<Self, anyhow::Error> {
        let mut cards = load_flashcards(dir)?;
        println!("Total fashcards: {}", cards.len());
        cards.sort_by_cached_key(|fs_card| {
            fs_card.card.last_reviewed.timestamp() + fs_card.card.review_after_secs
        });
        Ok(Self {
            sorted_cards: cards,
        })
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        for fs_card in &self.sorted_cards {
            let toml = toml::to_string(&fs_card.card).context("Failed to serialize card")?;
            // TODO: mkdir if necessary
            // TODO: check if file name does not exist
            fs::write(&fs_card.filename, toml).context("Failed to write card to disk")?;
        }
        Ok(())
    }

    pub fn add(&mut self, card: Flashcard) {
        let fname = card
            .question
            .split(' ')
            .take(3)
            .map(|word| word.to_string())
            .collect::<Vec<String>>()
            .join("_");
        let filename = format!("flashcards/{}/{}.toml", card.topic, fname);
        self.sorted_cards.push(CardFromFileSys { card, filename });
    }

    pub fn next(&self) -> Option<Flashcard> {
        self.sorted_cards
            .iter()
            .filter(|fs_card| {
                fs_card.card.last_reviewed.timestamp() + fs_card.card.review_after_secs
                    <= Utc::now().timestamp()
            })
            .next()
            .map(|fs_card| fs_card.card.clone())
    }

    pub fn ok(&mut self, card: Ulid) {
        if let Some(fs_card) = self
            .sorted_cards
            .iter_mut()
            .find(|fs_card| fs_card.card.id == card)
        {
            fs_card.card.review_after_secs = max(fs_card.card.review_after_secs, 86400) * 2;
            fs_card.card.last_reviewed = Utc::now();
        }
    }

    pub fn fail(&mut self, card: Ulid) {
        if let Some(fs_card) = self
            .sorted_cards
            .iter_mut()
            .find(|fs_card| fs_card.card.id == card)
        {
            // Don't prompt to review immediately.
            fs_card.card.review_after_secs = 3600;
            fs_card.card.last_reviewed = Utc::now();
        }
    }
}

fn load_flashcards(dir: &str) -> Result<Vec<CardFromFileSys>, anyhow::Error> {
    let pattern = format!("{}/**/*.toml", dir);
    Ok(glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok) // Filter out errors
        .map(|path| path.to_string_lossy().to_string()) // Convert paths to strings
        .map(|path| {
            let contents = fs::read_to_string(&path).expect("Failed to read file");
            let mut card: Flashcard = toml::from_str(&contents)
                .expect(format!("Failed to parse TOML: {}", path).as_str());
            card.topic = path
                .split('/')
                .rev()
                .nth(1)
                .unwrap_or_else(|| "unknown")
                .to_string();
            CardFromFileSys {
                card,
                filename: path,
            }
        })
        .collect())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Flashcard {
    #[serde(default = "Ulid::new")]
    pub id: Ulid,
    pub question: String,
    pub answer: String,

    pub examples: Vec<String>,

    pub source: Option<String>,
    pub img: Option<PathBuf>,

    // Each flashcard belongs to some topic: spanish, programming, maths, etc.
    #[serde(default)]
    #[serde(skip_serializing)]
    pub topic: String,

    pub last_reviewed: DateTime<Utc>,
    pub review_after_secs: i64,
}

#[derive(Debug)]
struct CardFromFileSys {
    card: Flashcard,
    filename: String,
}
