use std::fmt;

use leptos::prelude::*;
use server_fn::codec::JsonEncoding;
#[cfg(feature = "ssr")]
use duckdb::Error as DuckdbError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AppError {
    DuckdbError(String),
    ServerFnError(ServerFnErrorErr),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DuckdbError(e) => write!(f, "{}", e),
            AppError::ServerFnError(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(feature = "ssr")]
impl From<DuckdbError> for AppError {
    fn from(e: DuckdbError) -> Self {
        AppError::DuckdbError(e.to_string())
    }
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;
    fn from_server_fn_error(e: ServerFnErrorErr) -> Self {
        AppError::ServerFnError(e)
    }
}
