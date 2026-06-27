#[cfg(feature = "ssr")]
use crate::db::Database;
#[cfg(not(feature = "ssr"))]
use crate::api::client::delete_card;
use crate::model::Flashcard;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::*;

#[server(GetAllCards, "/api")]
pub async fn get_all_cards(tag: Option<String>) -> Result<Vec<Flashcard>, ServerFnError> {
    let db = Database::get_instance().unwrap().lock().unwrap();
    db.all_cards(tag)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn ListCards() -> impl IntoView {
    let (cards, set_cards) = signal(Vec::<Flashcard>::new());

    let fetch = move || {
        spawn_local(async move {
            match get_all_cards(None).await {
                Ok(fetched_cards) => set_cards.set(fetched_cards),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to fetch cards: {}", e).into())
                }
            }
        });
    };

    Effect::new(move |_| fetch());

    let on_delete = move |id: i64| {
        spawn_local(async move {
            #[cfg(not(feature = "ssr"))]
            match delete_card(id).await {
                Ok(_) => set_cards.update(|cs| cs.retain(|c| c.id != id)),
                Err(e) => web_sys::console::error_1(&e.to_string().into()),
            }
        });
    };

    view! {
        <div class="container mx-auto p-4">
            {move || {
                let cards = cards.get();
                view! {
                    <>
                        <h1 class="text-2xl font-bold mb-4">{"Total: "}{cards.len()}</h1>
                        <div class="overflow-x-auto">
                            <table class="min-w-full bg-white border border-gray-300">
                                <thead>
                                    <tr class="bg-gray-100">
                                        <th class="px-4 py-2 border">"Question"</th>
                                        <th class="px-4 py-2 border">"Answer"</th>
                                        <th class="px-4 py-2 border">"Tags"</th>
                                        <th class="px-4 py-2 border">"Last Reviewed"</th>
                                        <th class="px-4 py-2 border"></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {cards
                                        .into_iter()
                                        .map(|card| {
                                            let id = card.id;
                                            view! {
                                                <tr class="hover:bg-gray-50">
                                                    <td class="px-4 py-2 border">
                                                        <a href=format!("/cards/{}", id)>
                                                            {card.question}
                                                        </a>
                                                    </td>
                                                    <td class="px-4 py-2 border">{card.answer}</td>
                                                    <td class="px-4 py-2 border">
                                                        {card.tags.join(", ")}
                                                    </td>
                                                    <td class="px-4 py-2 border">
                                                        {card.last_reviewed.format("%Y-%m-%d %H:%M").to_string()}
                                                    </td>
                                                    <td class="px-4 py-2 border text-center">
                                                        <button
                                                            class="text-gray-400 hover:text-red-500 transition"
                                                            title="Delete card"
                                                            on:click=move |_| on_delete(id)
                                                        >
                                                            "🗑"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </tbody>
                            </table>
                        </div>
                    </>
                }
            }}
        </div>
    }
}
