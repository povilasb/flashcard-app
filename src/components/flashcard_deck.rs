use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::model;
#[cfg(feature = "ssr")]
use crate::db::Database;
use crate::components::review_by_tag::ReviewCards;
use crate::components::error_notification::ErrorNotification;


#[server(GetNextCards, "/api")]
async fn get_cards() -> Result<Vec<model::Flashcard>, ServerFnError> {
    let db = Database::get_instance("flashcards.db").unwrap();
    let db = db.lock().unwrap();
    db.cards_to_review().map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(SubmitAnswer, "/api")]
pub async fn submit_answer(card_id: i64, remembered: bool) -> Result<(), ServerFnError> {
    let db = Database::get_instance("flashcards.db").unwrap();
    let db = db.lock().unwrap();

    if remembered {
        db.ok(card_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    } else {
        db.fail(card_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    Ok(())
} 

#[component]
pub fn FlashcardDeck() -> impl IntoView {
    let (cards, set_cards) = signal(Vec::<model::Flashcard>::new());
    let (error, set_error) = signal(None::<String>);

    // Load cards
    Effect::new(move |_| {
        spawn_local(async move {
            match get_cards().await {
                Ok(loaded_cards) => {
                    set_cards.set(loaded_cards);
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load cards:\n {}", e)));
                }
            }
        });
    });


    view! {
        <ReviewCards cards=cards />
        <ErrorNotification error=error />
    }
} 