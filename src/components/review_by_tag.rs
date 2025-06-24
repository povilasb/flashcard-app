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

#[server(GetCardsByTag, "/api")]
async fn get_cards_by_tag(tag: String) -> Result<Vec<model::Flashcard>, ServerFnError> {
    let db = Database::get_instance("flashcards.db").unwrap();
    let db = db.lock().unwrap();
    db.all_cards(Some(tag)).map_err(ServerFnError::new)
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
    let (cards, set_cards) = signal(Vec::<model::Flashcard>::new());

    // Load cards
    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(loaded_cards) = get_cards_by_tag(tag()).await {
                set_cards.set(loaded_cards);
            }
            // TODO: error handling
        });
    });

    view! { <ReviewCards cards=cards /> }
} 

#[component]
pub fn ReviewCards(
    #[prop(into)] cards: Signal<Vec<model::Flashcard>>,
) -> impl IntoView {
    let (current_index, set_current_index) = signal(0usize);

    let handle_answer = Callback::new(move |answer: FlashcardAnswer| {
        let remembered = matches!(answer, FlashcardAnswer::Remember);
        spawn_local(async move {
            if let Some(card) = cards.get().get(current_index.get()) {
                let _ = submit_answer(card.id, remembered).await;
                set_current_index.update(|i| *i = *i + 1);
            }
        });
    });

    view! {
        <div class="review-cards">
            <progress
                class="w-full h-2.5 rounded-full"
                value={move || {
                    let total = cards.get().len();
                    if total == 0 { return 0; }
                    current_index.get() + 1
                }}
                max={move || cards.get().len()}
            ></progress>
            <Show
                when=move || {
                    cards.get().get(current_index.get()).is_some()
                }
                fallback=|| {
                    view! {
                        <div class="max-w-[600px] mx-auto my-8 p-4">
                            <div>"Done"</div>
                        </div>
                    }
                }
            >
                {move || {
                    let card = cards.get().get(current_index.get()).cloned().unwrap();
                    view! { <Flashcard card=card.clone() on_answer=handle_answer /> }
                }}
            </Show>
        </div>
    }
}