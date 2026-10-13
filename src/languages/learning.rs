//! Shared vocabulary and translation operations for HTTP and Leptos callers.

use super::{db::Database, model::Word};
use crate::{
    errors::AppError,
    settings::{Language, Settings},
};
use std::{future::Future, time::Duration};
use translators::{GoogleTranslator, Translator};

pub fn vocabulary() -> Result<Vec<Word>, AppError> {
    with_database(|db| db.all_words())
}

pub fn add_word(word: &str, translation: &str) -> Result<(), AppError> {
    validate_text(word, 256)?;
    if translation.len() > 4096 {
        return Err(AppError::InvalidInput("Translation is too long".into()));
    }
    with_database(|db| db.add_word(word.trim(), translation))
}

pub fn update_word(word: &str, translation: &str) -> Result<(), AppError> {
    validate_text(word, 256)?;
    if translation.len() > 4096 {
        return Err(AppError::InvalidInput("Translation is too long".into()));
    }
    with_database(|db| db.update_word_translation(word, translation))
}

pub fn delete_word(word: &str) -> Result<(), AppError> {
    validate_text(word, 256)?;
    with_database(|db| db.delete_word(word))
}

pub fn lookup(word: &str) -> Result<Option<String>, AppError> {
    validate_text(word, 256)?;
    with_database(|db| db.get_translation(word))
        .map(|value| value.filter(|text| !text.trim().is_empty()))
}

pub fn with_database<T>(
    operation: impl FnOnce(&mut Database) -> rusqlite::Result<T>,
) -> Result<T, AppError> {
    let settings = Settings::get();
    let mut db = Database::get_instance(&settings.db_path, settings.learning_language.as_str())?
        .lock()
        .map_err(|_| AppError::SqliteError("Vocabulary database unavailable".into()))?;
    operation(&mut db).map_err(Into::into)
}

pub async fn translate(text: &str) -> Result<String, AppError> {
    validate_text(text, 16_384)?;
    let source = match Settings::get().learning_language {
        Language::Spanish => "es",
        Language::French => "fr",
        Language::Portuguese => "pt-PT",
        Language::German => "de",
    };
    let translator = GoogleTranslator::default();
    let translated = provider_timeout(async {
        translator
            .translate_async(text, source, "en")
            .await
            .map_err(AppError::from)
    })
    .await?;
    if translated.trim().is_empty() {
        return Err(AppError::GoogleTranslateError("Empty translation".into()));
    }
    Ok(translated)
}

pub async fn provider_timeout<T>(
    operation: impl Future<Output = Result<T, AppError>>,
) -> Result<T, AppError> {
    tokio::time::timeout(Duration::from_secs(60), operation)
        .await
        .map_err(|_| AppError::Timeout)?
}

fn validate_text(text: &str, max: usize) -> Result<(), AppError> {
    if text.trim().is_empty() || text.len() > max {
        return Err(AppError::InvalidInput(format!(
            "Text must contain 1–{max} bytes"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn provider_calls_are_bounded() {
        let result: Result<(), AppError> = provider_timeout(std::future::pending()).await;

        assert!(matches!(result, Err(AppError::Timeout)));
    }

    #[test]
    fn rejects_blank_and_oversized_text() {
        assert!(validate_text(" ", 256).is_err());
        assert!(validate_text(&"x".repeat(257), 256).is_err());
        assert!(validate_text("café", 256).is_ok());
    }
}
