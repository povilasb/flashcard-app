mod flashcard;

use flashcard::Database;
use leptos::*;
use std::error::Error;

#[component]
fn Flashcard<'a>(card: flashcard::ReviewCard<'a>) -> impl IntoView {
    let (visible, set_visible) = create_signal(false);
    let toggle_visibility = move |_| {
        set_visible.update(|v| *v = !*v);
    };
    let question = card.question().to_string();
    let answer = card.answer().to_string();
    view! {
        <div id="card-view">
            <div class="flashcard">
                <h3 class="question">Q: {question}</h3>
                <p id="answer" style:display=move || { if visible.get() { "block" } else { "none" }} >A: {answer}</p>
            </div>
            <button class="show-answer-btn" on:click=toggle_visibility >Show Answer</button>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let mut db = Database::load(DB_DIR).unwrap();
    let card = db.review().next().unwrap();
    view! {
        <Flashcard card=card />
    }
}

const DB_DIR: &str = "flashcards";

fn main() -> Result<(), Box<dyn Error>> {
    Ok(mount_to_body(move || view! { <App /> }))
}
