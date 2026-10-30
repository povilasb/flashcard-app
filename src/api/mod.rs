//! HTTP request types, shared database access, and response error mapping.

pub mod cards;
pub mod languages;
pub mod media;
pub mod rich_text;

use crate::db::Database;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::{error::Error, sync::MutexGuard};

#[derive(Debug)]
pub enum ApiError {
    BadRequest(&'static str),
    NotFound,
    Internal(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitAnswerRequest {
    pub remembered: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCardRequest {
    pub question: String,
    pub answer: String,
    pub examples: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub answer_img_fname: Option<String>,
    pub question_img_fname: Option<String>,
}

pub(crate) fn database() -> Result<MutexGuard<'static, Database>, ApiError> {
    Database::get_instance()?
        .lock()
        .map_err(|error| ApiError::Internal(error.to_string()))
}

impl From<Box<dyn Error>> for ApiError {
    fn from(error: Box<dyn Error>) -> Self {
        match error.downcast_ref::<rusqlite::Error>() {
            Some(rusqlite::Error::QueryReturnedNoRows) => Self::NotFound,
            _ => Self::Internal(error.to_string()),
        }
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::QueryReturnedNoRows => Self::NotFound,
            _ => Self::Internal(error.to_string()),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        Self::Internal(error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            Self::NotFound => StatusCode::NOT_FOUND.into_response(),
            Self::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message).into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_row_maps_to_not_found() {
        let error: Box<dyn Error> = Box::new(rusqlite::Error::QueryReturnedNoRows);

        let response = ApiError::from(error).into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let response = ApiError::from(rusqlite::Error::QueryReturnedNoRows).into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn bad_request_preserves_validation_message() {
        let response = ApiError::BadRequest("Question is required").into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"Question is required");
    }

    #[tokio::test]
    async fn other_database_errors_preserve_internal_error_status_and_message() {
        for error in [
            ApiError::from(Box::new(rusqlite::Error::InvalidQuery) as Box<dyn Error>),
            ApiError::from(rusqlite::Error::InvalidQuery),
            ApiError::from(anyhow::anyhow!(rusqlite::Error::InvalidQuery)),
        ] {
            let response = error.into_response();

            assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
            let body = axum::body::to_bytes(response.into_body(), 1024)
                .await
                .unwrap();
            assert_eq!(
                body.as_ref(),
                rusqlite::Error::InvalidQuery.to_string().as_bytes()
            );
        }
    }
}
