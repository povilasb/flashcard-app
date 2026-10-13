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
