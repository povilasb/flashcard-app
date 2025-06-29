//! This module provides utilities for learning languages.
//! Which later leverages flashcards.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[cfg(feature = "ssr")]
mod db;
#[cfg(feature = "ssr")]
mod ai;
pub mod components;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Word {
    pub word: String,
    pub translation: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A sentence with a new word and its translation for iterative language learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSentence {
    pub text: String,
    pub new_word: String,
    pub translation: String,
}