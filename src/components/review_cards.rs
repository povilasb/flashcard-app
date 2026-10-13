use crate::components::error_notification::ErrorNotification;
use crate::components::flashcard::Flashcard;
#[cfg(feature = "hydrate")]
use crate::api::client::{fetch_cards_to_review, submit_answer};
use crate::model;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Review all cards that are due for review.
#[component]
pub fn ReviewAllCards() -> impl IntoView {
    let (cards, set_cards) = signal(Vec::<model::Flashcard>::new());
    let (error, set_error) = signal(None::<String>);

    // Load cards
    Effect::new(move |_| {
        spawn_local(async move {
            #[cfg(feature = "hydrate")]
            match fetch_cards_to_review().await {
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

/// A reusable component to review a given list of cards.
#[component]
pub fn ReviewCards(#[prop(into)] cards: Signal<Vec<model::Flashcard>>) -> impl IntoView {
    let current_index = RwSignal::new(0usize);
    let (error, set_error) = signal(None::<String>);

    let handle_answer = Callback::new(move |answer: model::FlashcardAnswer| {
        let remembered = matches!(answer, model::FlashcardAnswer::Remember);
        spawn_local(async move {
            #[cfg(feature = "hydrate")]
            if let Some(card) = cards.get().get(current_index.get()) {
                match submit_answer(card.id, remembered).await {
                    Ok(_) => {
                        current_index.set(current_index.get() + 1);
                        set_error.set(None);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Failed to submit answer:\n{}", e)));
                    }
                }
            }
        });
    });

    view! {
        <div class="review-cards">
            <progress
                class="w-full h-2.5 rounded-full"
                value=move || {
                    let total = cards.get().len();
                    if total == 0 {
                        return 0;
                    }
                    current_index.get() + 1
                }
                max=move || cards.get().len()
            ></progress>
            <Show
                when=move || { cards.get().get(current_index.get()).is_some() }
                fallback=|| {
                    view! {
                        <div class="max-w-[600px] mx-auto my-8 p-4">
                            <div class="text-center">"Nothing to review at the moment."</div>
                        </div>
                    }
                }
            >
                {move || {
                    let card = cards.get().get(current_index.get()).cloned().unwrap();
                    view! { <Flashcard card=card.clone() on_answer=handle_answer /> }
                }}
            </Show>
            <ErrorNotification error=error />
        </div>
    }
}
