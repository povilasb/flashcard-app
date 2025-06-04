use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::model::{FlashcardAnswer};
use crate::components::flashcard::Flashcard;
use crate::model;
#[cfg(feature = "ssr")]
use crate::db::Database;


#[server(GetNextCard, "/api")]
pub async fn get_next_card() -> Result<Option<model::Flashcard>, ServerFnError> {
    let db = Database::get_instance("../flashcards").unwrap();
    let mut db = db.lock().unwrap();
    let card = db.next();
    Ok(card)
}

#[server(SubmitAnswer, "/api")]
pub async fn submit_answer(card_id: String, remembered: bool) -> Result<(), ServerFnError> {
    let db = Database::get_instance("../flashcards").unwrap();
    let mut db = db.lock().unwrap();

    if remembered {
        db.ok(card_id);
    } else {
        db.fail(card_id);
    }
    db.save().unwrap();

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
                        fallback=|| view! { <div>"Loading..."</div> }
                    >
                        {move || {
                            let card = current_card.get().unwrap();
                            view! {
                                <Flashcard
                                    card=card
                                    on_answer=handle_answer
                                />
                            }
                        }}
                    </Show>
                }
            }}
        </div>
    }
} 