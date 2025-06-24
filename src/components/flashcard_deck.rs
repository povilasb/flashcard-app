use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::model::{FlashcardAnswer};
use crate::components::flashcard::Flashcard;
use crate::model;
#[cfg(feature = "ssr")]
use crate::db::Database;


#[server(GetNextCard, "/api")]
async fn get_next_card() -> Result<Option<model::Flashcard>, ServerFnError> {
    let db = Database::get_instance("flashcards.db").unwrap();
    let db = db.lock().unwrap();
    let card = db.next().map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(card)
}

#[server(SubmitAnswer, "/api")]
pub async fn submit_answer(card_id: i64, remembered: bool) -> Result<(), ServerFnError> {
    let db = Database::get_instance("flashcards.db").unwrap();
    let db = db.lock().unwrap();

    return Err(ServerFnError::new("Not implemented".to_string()));

    if remembered {
        db.ok(card_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    } else {
        db.fail(card_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    Ok(())
} 

#[component]
pub fn FlashcardDeck() -> impl IntoView {
    let (current_card, set_current_card) = signal(None::<model::Flashcard>);

    // Load initial card
    Effect::new(move |_| {
        spawn_local(async move {
            let result = get_next_card().await;
            if let Ok(card) = result {
                set_current_card.set(card);
            }
        });
    });

    let handle_answer = Callback::new(move |answer: FlashcardAnswer| {
        let remembered = matches!(answer, FlashcardAnswer::Remember);
        spawn_local(async move {
            if let Some(card) = current_card.get() {
                let _ = submit_answer(card.id, remembered).await;
                let result = get_next_card().await;
                if let Ok(card) = result {
                    set_current_card.set(card);
                }
            }
        });
    });

    view! {
        <div class="flashcard-deck">
            {move || {
                view! {
                    <Show
                        when=move || current_card.get().is_some()
                        fallback=|| {
                            view! {
                                <div class="max-w-[600px] mx-auto my-8 p-4">
                                    <div>"No more cards to review at the moment."</div>
                                </div>
                            }
                        }
                    >
                        {move || {
                            let card = current_card.get().unwrap();
                            view! { <Flashcard card=card on_answer=handle_answer /> }
                        }}
                    </Show>
                }
            }}
        </div>
    }
} 