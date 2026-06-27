use crate::components::flashcard::Flashcard;
use crate::model;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::*;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;

#[cfg(not(feature = "ssr"))]
use crate::api::client::fetch_card;

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
            #[cfg(not(feature = "ssr"))]
            match fetch_card(id()).await {
                Ok(card) => set_card.set(Some(card)),
                Err(e) => web_sys::console::error_1(&e.to_string().into()),
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
