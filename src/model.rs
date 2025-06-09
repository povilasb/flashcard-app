//! Core models for the flashcard app.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Flashcard {
    pub id: i64,
    pub question: String,
    pub answer: String,

    pub examples: Option<String>,
    pub source: Option<String>,
    pub img: Option<String>,
    pub tags: Vec<String>,

    pub last_reviewed: DateTime<Utc>,
    pub review_after_secs: i64,
}

impl Flashcard {
    pub fn new(question: String, answer: String) -> Self {
        Self {
            id: 0,
            question,
            answer,
            examples: None,
            source: None,
            img: None,
            tags: Vec::new(),
            last_reviewed: Utc::now(),
            review_after_secs: 43200, // 12 hours
        }
    }
}

#[derive(Copy, Clone)]
pub enum FlashcardAnswer {
    Remember,
    Not,
}