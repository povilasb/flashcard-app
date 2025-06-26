//! This module provides utilities for learning languages.
//! Which later leverages flashcards.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[cfg(feature = "ssr")]
mod db;
pub mod components;

#[cfg(feature = "ssr")]
pub use db::Database;


#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Word {
    pub word: String,
    pub translation: Option<String>,
    pub created_at: DateTime<Utc>,
}