use crate::components::edit_card::get_card;
use crate::components::flashcard::Flashcard;
use crate::model;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[derive(Params, PartialEq, Clone)]
struct ViewCardParams {
    id: Option<i64>,
}

#[component]
pub fn ViewCard() -> impl IntoView {
    let (card, set_card) = signal(None::<model::Flashcard>);

    let params = use_params::<ViewCardParams>();
    let id = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|params| params.id)
            .unwrap_or_default()
    };

    // Load card data
    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(fetched_card) = get_card(id()).await {
                set_card.set(Some(fetched_card));
            } else {
                web_sys::console::error_1(&"Failed to fetch card".into());
            }
        });
    });

    view! {
        <Show
            when=move || card.get().is_some()
            fallback=|| {
                view! {
                    <div class="max-w-[600px] mx-auto my-8 p-4">
                        <div>"Loading..."</div>
                    </div>
                }
            }
        >
            {move || {
                let card = card.get().unwrap();
                view! { <Flashcard card=card on_answer=Callback::new(move |_| {}) /> }
            }}
        </Show>
    }
}
