//! Flashcard HTTP endpoints preserving card data and review schedules.

use axum::{extract::Path, http::StatusCode, Json};
use serde::Deserialize;

use crate::{
    api::{database, ApiError, CreateCardRequest, SubmitAnswerRequest},
    model::Flashcard,
};

#[derive(Deserialize)]
pub struct ListCardsQuery {
    pub tag: Option<String>,
}

pub async fn submit_answer(
    Path(id): Path<i64>,
    Json(req): Json<SubmitAnswerRequest>,
) -> Result<StatusCode, ApiError> {
    let mut db = database()?;

    if req.remembered {
        db.ok(id)?;
    } else {
        db.fail(id)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_card(
    Path(id): Path<i64>,
    Json(req): Json<CreateCardRequest>,
) -> Result<StatusCode, ApiError> {
    if req.question.trim().is_empty() {
        return Err(ApiError::BadRequest("Question is required"));
    }

    let mut db = database()?;
    let mut card = db.get_card(id)?;

    card.question = req.question;
    card.answer = req.answer;
    card.examples = req.examples;
    card.source = req.source;
    card.question_img = req.question_img_fname;
    card.img = req.answer_img_fname;
    card.tags = req
        .tags
        .into_iter()
        .map(|tag| tag.trim().to_owned())
        .filter(|tag| !tag.is_empty())
        .collect();
    card.tags.sort();
    card.tags.dedup();

    db.update_card(&card)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_card(Json(req): Json<CreateCardRequest>) -> Result<StatusCode, ApiError> {
    let mut card = Flashcard::new(req.question, req.answer);
    card.examples = req.examples;
    card.source = req.source;
    card.tags = req.tags;
    card.img = req.answer_img_fname;
    card.question_img = req.question_img_fname;

    database()?.add_card(&card)?;

    Ok(StatusCode::CREATED)
}
