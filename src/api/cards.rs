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

pub async fn get_all_cards(Query(query): Query<ListCardsQuery>) -> impl IntoResponse {
    let db = Database::get_instance().unwrap().lock().unwrap();
    match db.all_cards(query.tag) {
        Ok(cards) => Json(cards).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_cards_to_review() -> impl IntoResponse {
    let db = Database::get_instance().unwrap().lock().unwrap();
    match db.cards_to_review() {
        Ok(cards) => Json(cards).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn submit_answer(
    Path(id): Path<i64>,
    Json(req): Json<SubmitAnswerRequest>,
) -> impl IntoResponse {
    let mut db = Database::get_instance().unwrap().lock().unwrap();
    let result = if req.remembered {
        db.ok(id)
    } else {
        db.fail(id)
    };

    match result {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_card(Path(id): Path<i64>) -> impl IntoResponse {
    let db = Database::get_instance().unwrap().lock().unwrap();
    match db.get_card(id) {
        Ok(card) => Json(card).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn delete_card(Path(id): Path<i64>) -> impl IntoResponse {
    let mut db = Database::get_instance().unwrap().lock().unwrap();
    match db.delete_card(id) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn update_card(Path(id): Path<i64>, Json(req): Json<CreateCardRequest>) -> impl IntoResponse {
    if req.question.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, "Question is required").into_response();
    }
    let mut db = Database::get_instance().unwrap().lock().unwrap();
    let mut card = match db.get_card(id) {
        Ok(card) => card,
        Err(error) => {
            return match error.downcast_ref::<rusqlite::Error>() {
                Some(rusqlite::Error::QueryReturnedNoRows) => StatusCode::NOT_FOUND.into_response(),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
            };
        }
    };
    card.question = req.question;
    card.answer = req.answer;
    card.examples = req.examples;
    card.source = req.source;
    card.question_img = req.question_img_fname;
    card.img = req.answer_img_fname;
    card.tags = req.tags.into_iter().map(|tag| tag.trim().to_owned())
        .filter(|tag| !tag.is_empty()).collect();
    card.tags.sort();
    card.tags.dedup();
    match db.update_card(&card) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response(),
    }
}

pub async fn create_card(Json(req): Json<CreateCardRequest>) -> impl IntoResponse {
    let mut db = Database::get_instance().unwrap().lock().unwrap();

    let mut card = Flashcard::new(req.question, req.answer);
    card.examples = req.examples;
    card.source = req.source;
    card.tags = req.tags;
    card.img = req.answer_img_fname;
    card.question_img = req.question_img_fname;

    match db.add_card(&card) {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
