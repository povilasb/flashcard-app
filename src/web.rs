mod flashcard;

use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use env_logger::Env;
use serde::Deserialize;

use flashcard::{Database, Flashcard};

const add_card_form: &str = include_str!("../add_card.html");

#[get("/")]
async fn add_flashcard_form() -> impl Responder {
    HttpResponse::Ok().body(add_card_form)
}

#[derive(Debug, Deserialize)]
struct AddFlashcard {
    topic: String,
    question: String,
    answer: String,
    examples: String,
    source: Option<String>,
}

#[post("/flashcard")]
async fn add_flashcard(q: web::Form<AddFlashcard>) -> impl Responder {
    let q = q.into_inner();
    let card = Flashcard {
        topic: q.topic,
        question: q.question,
        answer: q.answer,
        examples: q
            .examples
            .replace('\r', "")
            .split('\n')
            .map(|s| s.to_string())
            .collect(),
        added: Utc::now().to_string(),
        last_reviewed: Utc::now(),
        source: q.source,
        review_after_secs: 0,
    };
    let mut db = Database::load("flashcards").expect("Failed to load database");
    db.add(card);
    db.save().expect("Failed to save database");
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(add_flashcard_form)
            .service(add_flashcard)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
