use std::fmt;

use leptos::prelude::*;
#[cfg(feature = "ssr")]
use rig::completion::PromptError;
#[cfg(feature = "ssr")]
use rusqlite::Error as SqliteError;
use serde::{Deserialize, Serialize};
use server_fn::codec::JsonEncoding;
#[cfg(feature = "ssr")]
use translators;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AppError {
    SqliteError(String),
    ServerFnError(ServerFnErrorErr),
    LlmError(String),
    GoogleTranslateError(String),
    Timeout,
    InvalidInput(String),
    NotFound,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::SqliteError(e) => write!(f, "{}", e),
            AppError::ServerFnError(e) => write!(f, "{}", e),
            AppError::LlmError(e) => write!(f, "{}", e),
            AppError::GoogleTranslateError(e) => write!(f, "{}", e),
            AppError::Timeout => write!(f, "Provider request timed out"),
            AppError::InvalidInput(e) => write!(f, "{}", e),
            AppError::NotFound => write!(f, "Word not found"),
        }
    }
}

#[cfg(feature = "ssr")]
impl From<SqliteError> for AppError {
    fn from(e: SqliteError) -> Self {
        match e {
            SqliteError::QueryReturnedNoRows => AppError::NotFound,
            _ => AppError::SqliteError(e.to_string()),
        }
    }
}

#[cfg(feature = "ssr")]
impl From<PromptError> for AppError {
    fn from(e: PromptError) -> Self {
        AppError::LlmError(e.to_string())
    }
}

#[cfg(feature = "ssr")]
impl From<translators::Error> for AppError {
    fn from(e: translators::Error) -> Self {
        AppError::GoogleTranslateError(e.to_string())
    }
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;
    fn from_server_fn_error(e: ServerFnErrorErr) -> Self {
        AppError::ServerFnError(e)
    }
}
