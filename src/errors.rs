use std::fmt;

#[cfg(feature = "ssr")]
use duckdb::Error as DuckdbError;
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use rig::completion::PromptError;
use serde::{Deserialize, Serialize};
use server_fn::codec::JsonEncoding;
#[cfg(feature = "ssr")]
use translators;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AppError {
    DuckdbError(String),
    ServerFnError(ServerFnErrorErr),
    LlmError(String),
    GoogleTranslateError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DuckdbError(e) => write!(f, "{}", e),
            AppError::ServerFnError(e) => write!(f, "{}", e),
            AppError::LlmError(e) => write!(f, "{}", e),
            AppError::GoogleTranslateError(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(feature = "ssr")]
impl From<DuckdbError> for AppError {
    fn from(e: DuckdbError) -> Self {
        AppError::DuckdbError(e.to_string())
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
