//! Core models for the flashcard app.

use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use ulid::Ulid;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Flashcard {
    // #[serde(default = "Ulid::new")]
    pub id: String,
    pub question: String,
    pub answer: String,

    pub examples: Vec<String>,

    pub source: Option<String>,
    pub img: Option<PathBuf>,
    #[serde(default)]
    pub tags: Vec<String>,

    pub last_reviewed: DateTime<Utc>,
    pub review_after_secs: i64,
}

#[cfg(feature = "ssr")]
impl Flashcard {
    pub fn new(question: String, answer: String) -> Self {
        Self {
            id: Ulid::new().to_string(),
            question,
            answer,
            examples: Vec::new(),
            source: None,
            img: None,
            tags: Vec::new(),
            last_reviewed: Utc::now(),
            review_after_secs: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub enum FlashcardAnswer {
    Remember,
    Not,
}