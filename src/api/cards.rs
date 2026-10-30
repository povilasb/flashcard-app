//! Flashcard HTTP endpoints preserving card data and review schedules.

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{
    api::{CreateCardRequest, SubmitAnswerRequest},
    db::Database,
    model::Flashcard,
};

#[derive(Deserialize)]
pub struct ListCardsQuery {
    pub tag: Option<String>,
}

pub async fn get_all_cards(
    Query(query): Query<ListCardsQuery>,
) -> Result<Json<Vec<Flashcard>>, (StatusCode, String)> {
    let db = Database::get_instance().unwrap().lock().unwrap();

    db.all_cards(query.tag)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_cards_to_review() -> Result<Json<Vec<Flashcard>>, (StatusCode, String)> {
    let db = Database::get_instance().unwrap().lock().unwrap();

    db.cards_to_review()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_random_cards() -> Result<Json<Vec<Flashcard>>, (StatusCode, String)> {
    let db = Database::get_instance().unwrap().lock().unwrap();

    db.random_cards()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn submit_answer(
    Path(id): Path<i64>,
    Json(req): Json<SubmitAnswerRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut db = Database::get_instance().unwrap().lock().unwrap();
    let result = if req.remembered {
        db.ok(id)
    } else {
        db.fail(id)
    };

    result
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_card(Path(id): Path<i64>) -> Result<Json<Flashcard>, StatusCode> {
    let db = Database::get_instance().unwrap().lock().unwrap();

    db.get_card(id).map(Json).map_err(|_| StatusCode::NOT_FOUND)
}

pub async fn delete_card(Path(id): Path<i64>) -> Result<StatusCode, axum::response::Response> {
    let mut db = Database::get_instance().unwrap().lock().unwrap();

    delete_card_from(&mut db, id)
}

fn delete_card_from(db: &mut Database, id: i64) -> Result<StatusCode, axum::response::Response> {
    db.delete_card(id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|error| match error.downcast_ref::<rusqlite::Error>() {
            Some(rusqlite::Error::QueryReturnedNoRows) => StatusCode::NOT_FOUND.into_response(),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_returns_no_content_then_not_found() {
        let mut db = Database::load_or_init(":memory:").unwrap();
        let mut card = Flashcard::new("question".into(), "answer".into());
        card.tags = vec!["rust".into()];
        db.add_card(&card).unwrap();
        db.ok(1).unwrap();

        let response = delete_card_from(&mut db, 1).into_response();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(db.all_cards(None).unwrap().is_empty());
        assert!(db.review_history().unwrap().is_empty());
        assert_eq!(
            delete_card_from(&mut db, 1).into_response().status(),
            StatusCode::NOT_FOUND
        );
    }
}

pub async fn update_card(
    Path(id): Path<i64>,
    Json(req): Json<CreateCardRequest>,
) -> Result<StatusCode, axum::response::Response> {
    if req.question.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Question is required").into_response());
    }

    let mut db = Database::get_instance().unwrap().lock().unwrap();
    let mut card =
        db.get_card(id)
            .map_err(|error| match error.downcast_ref::<rusqlite::Error>() {
                Some(rusqlite::Error::QueryReturnedNoRows) => StatusCode::NOT_FOUND.into_response(),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
            })?;

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

    db.update_card(&card)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response())
}

pub async fn create_card(
    Json(req): Json<CreateCardRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut db = Database::get_instance().unwrap().lock().unwrap();

    let mut card = Flashcard::new(req.question, req.answer);
    card.examples = req.examples;
    card.source = req.source;
    card.tags = req.tags;
    card.img = req.answer_img_fname;
    card.question_img = req.question_img_fname;

    db.add_card(&card)
        .map(|_| StatusCode::CREATED)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
