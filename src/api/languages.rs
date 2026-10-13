//! JSON endpoints for the configured learning language.
use crate::{
    errors::AppError,
    languages::{ai::Agent, learning},
    settings::Language,
};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct WordInput {
    word: String,
    translation: String,
}
#[derive(Deserialize)]
pub struct WordQuery {
    word: String,
}
#[derive(Deserialize)]
pub struct TextInput {
    text: String,
}
#[derive(Serialize)]
struct Translation {
    translation: Option<String>,
}
#[derive(Serialize)]
struct Sentence {
    sentence: crate::languages::model::NewSentence,
    language: Language,
}
#[derive(Serialize)]
struct Story {
    story: String,
    language: Language,
}

pub async fn vocabulary() -> Response {
    respond(learning::vocabulary().map(Json))
}
pub async fn add_word(Json(input): Json<WordInput>) -> Response {
    respond(learning::add_word(&input.word, &input.translation).map(|_| StatusCode::CREATED))
}
pub async fn update_word(Json(input): Json<WordInput>) -> Response {
    respond(learning::update_word(&input.word, &input.translation).map(|_| StatusCode::NO_CONTENT))
}
pub async fn delete_word(Query(input): Query<WordQuery>) -> Response {
    respond(learning::delete_word(&input.word).map(|_| StatusCode::NO_CONTENT))
}
pub async fn import() -> Response {
    respond(
        Agent::from_settings()
            .populate_words_db()
            .await
            .map(|_| StatusCode::NO_CONTENT),
    )
}
pub async fn sentence() -> Response {
    let agent = Agent::from_settings();
    respond(agent.gen_new_sentence().await.map(|sentence| {
        Json(Sentence {
            sentence,
            language: agent.lang,
        })
    }))
}
pub async fn story() -> Response {
    let agent = Agent::from_settings();
    respond(agent.gen_story().await.map(|story| {
        Json(Story {
            story,
            language: agent.lang,
        })
    }))
}
pub async fn lookup(Query(input): Query<WordQuery>) -> Response {
    respond(learning::lookup(&input.word).map(|translation| Json(Translation { translation })))
}
pub async fn translate(Json(input): Json<TextInput>) -> Response {
    respond(learning::translate(&input.text).await.map(|translation| {
        Json(Translation {
            translation: Some(translation),
        })
    }))
}

fn respond(result: Result<impl IntoResponse, AppError>) -> Response {
    match result {
        Ok(value) => value.into_response(),
        Err(error) => {
            let (status, message) = match &error {
                AppError::InvalidInput(message) => (StatusCode::BAD_REQUEST, message.as_str()),
                AppError::NotFound => (StatusCode::NOT_FOUND, "Word not found"),
                AppError::Timeout => (
                    StatusCode::GATEWAY_TIMEOUT,
                    "Provider request timed out. Please retry.",
                ),
                AppError::LlmError(_) | AppError::GoogleTranslateError(_) => (
                    StatusCode::BAD_GATEWAY,
                    "Language provider failed. Please retry.",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Could not access vocabulary.",
                ),
            };
            log::error!("Language request failed: {error}");
            #[derive(Serialize)]
            struct ErrorBody<'a> {
                error: ErrorMessage<'a>,
            }
            #[derive(Serialize)]
            struct ErrorMessage<'a> {
                message: &'a str,
            }
            (
                status,
                Json(ErrorBody {
                    error: ErrorMessage { message },
                }),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_invalid_payload_before_accessing_database_and_hides_provider_details() {
        let response = add_word(Json(WordInput {
            word: "  ".into(),
            translation: "".into(),
        }))
        .await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response = respond(Err::<StatusCode, _>(AppError::LlmError(
            "secret provider details".into(),
        )));
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();

        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            "{\"error\":{\"message\":\"Language provider failed. Please retry.\"}}"
        );
    }

    #[test]
    fn maps_language_errors_to_http_statuses() {
        for (error, status) in [
            (AppError::NotFound, StatusCode::NOT_FOUND),
            (
                AppError::InvalidInput("empty".into()),
                StatusCode::BAD_REQUEST,
            ),
            (
                AppError::LlmError("private provider details".into()),
                StatusCode::BAD_GATEWAY,
            ),
            (AppError::Timeout, StatusCode::GATEWAY_TIMEOUT),
            (
                AppError::SqliteError("private path".into()),
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ] {
            let response = respond(Err::<StatusCode, _>(error));

            assert_eq!(response.status(), status);
        }
    }
}
