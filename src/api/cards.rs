use axum::{extract::Path, http::StatusCode, response::IntoResponse, Json};

use crate::{
    api::CreateCardRequest,
    db::Database,
    model::Flashcard,
};

pub async fn get_card(Path(id): Path<i64>) -> impl IntoResponse {
    let db = Database::get_instance().unwrap().lock().unwrap();
    match db.get_card(id) {
        Ok(card) => Json(card).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn delete_card(Path(id): Path<i64>) -> impl IntoResponse {
    let db = Database::get_instance().unwrap().lock().unwrap();
    match db.delete_card(id) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn create_card(Json(req): Json<CreateCardRequest>) -> impl IntoResponse {
    let db = Database::get_instance().unwrap().lock().unwrap();

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
