
use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::params::Params;
use leptos_router::hooks::use_params;
use crate::model::{FlashcardAnswer};
use crate::components::flashcard::Flashcard;
use crate::model;
use crate::components::flashcard_deck::submit_answer;
#[cfg(feature = "ssr")]
use crate::db::Database;

#[server(GetNextCardByTag, "/api")]
async fn get_next_card_by_tag(tag: String) -> Result<Option<model::Flashcard>, ServerFnError> {
    let db = Database::get_instance("flashcards.db").unwrap();
    let db = db.lock().unwrap();
    let card = db.next_by_tag(&tag).map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(card)
}

#[derive(Params, PartialEq, Clone)]
struct ReviewByTagParams {
    tag: Option<String>,
}

#[component]
pub fn ReviewByTag() -> impl IntoView {
    let params = use_params::<ReviewByTagParams>();
    let tag = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.tag.clone())
            .unwrap()
    };
    let (current_card, set_current_card) = signal(None::<model::Flashcard>);

    // Load first card
    Effect::new(move |_| {
        spawn_local(async move {
            let result = get_next_card_by_tag(tag()).await;
            // TODO: error handling
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
                let result = get_next_card_by_tag(tag()).await;
                // TODO: error handling
                if let Ok(card) = result {
                    set_current_card.set(card);
                }
            }
        });
    });

    view! {
        <div>
            {move || {
                view! {
                    <Show
                        when=move || current_card.get().is_some()
                        fallback=|| {
                            view! {
                                <div class="max-w-[600px] mx-auto my-8 p-4">
                                    <div>"No cards for tag."</div>
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