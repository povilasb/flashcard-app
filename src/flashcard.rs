use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::fs;

#[derive(Debug)]
pub struct Database {
    sorted_cards: Vec<CardFromFileSys>,
}

impl Database {
    pub fn load(dir: &str) -> Result<Self, anyhow::Error> {
        let mut cards = load_flashcards(dir)?;
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
            fs::write(&fs_card.filename, toml).context("Failed to write card to disk")?;
        }
        Ok(())
    }

    pub fn review(&mut self) -> impl Iterator<Item = ReviewCard> {
        self.sorted_cards
            .iter_mut()
            .take_while(|fs_card| {
                fs_card.card.last_reviewed.timestamp() + fs_card.card.review_after_secs
                    <= Utc::now().timestamp()
            })
            .map(|fs_card| ReviewCard {
                card: &mut fs_card.card,
            })
    }
}

pub struct ReviewCard<'a> {
    card: &'a mut Flashcard,
}

impl<'a> ReviewCard<'a> {
    pub fn question(&self) -> &str {
        &self.card.question
    }

    pub fn answer(&self) -> &str {
        &self.card.answer
    }

    pub fn ok(&mut self) {
        self.card.review_after_secs = max(self.card.review_after_secs, 86400) * 2;
        self.card.last_reviewed = Utc::now();
    }

    pub fn fail(&mut self) {
        self.card.review_after_secs = 0;
        self.card.last_reviewed = Utc::now();
    }
}

fn load_flashcards(dir: &str) -> Result<Vec<CardFromFileSys>, anyhow::Error> {
    let pattern = format!("{}/**/*.toml", dir);
    Ok(glob(&pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok) // Filter out errors
        .map(|path| path.to_string_lossy().to_string()) // Convert paths to strings
        .filter(|path| !path.ends_with("boxes.toml")) // Filter out directories
        .map(|path| {
            let contents = fs::read_to_string(&path).expect("Failed to read file");
            let card: Flashcard = toml::from_str(&contents)
                .expect(format!("Failed to parse TOML: {}", path).as_str());
            CardFromFileSys {
                card,
                filename: path,
            }
        })
        .collect())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Flashcard {
    pub question: String,
    pub answer: String,
    pub examples: Vec<String>,

    pub added: String,
    pub last_reviewed: DateTime<Utc>,
    pub review_after_secs: i64,
    pub source: Option<String>,
}

#[derive(Debug)]
struct CardFromFileSys {
    card: Flashcard,
    filename: String,
}
