use leptos::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
#[cfg(feature = "ssr")]
use crate::db::Database;
use crate::model::Flashcard;

#[server(GetAllCards, "/api")]
pub async fn get_all_cards(tag: Option<String>) -> Result<Vec<Flashcard>, ServerFnError> {
    let db = Database::get_instance("flashcards.db").map_err(|e| ServerFnError::new(e.to_string()))?;
    let db = db.lock().unwrap();
    db.all_cards(tag).map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn ListCards() -> impl IntoView {
    let (cards, set_cards) = signal(Vec::new());
    Effect::new(move |_| {
        spawn_local(async move {
            match get_all_cards(None).await {
                Ok(fetched_cards) => set_cards.set(fetched_cards),
                Err(e) => web_sys::console::error_1(&format!("Failed to fetch cards: {}", e).into()),
            }
        });
    });

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
                                    </tr>
                                </thead>
                                <tbody>
                                    {cards
                                        .into_iter()
                                        .map(|card| {
                                            view! {
                                                <tr class="hover:bg-gray-50 cursor-pointer">
                                                    <td class="px-4 py-2 border">
                                                        <a href=format!(
                                                            "/cards/{}",
                                                            card.id.to_string(),
                                                        )>{card.question}</a>
                                                    </td>
                                                    <td class="px-4 py-2 border">{card.answer}</td>
                                                    <td class="px-4 py-2 border">{card.tags.join(", ")}</td>
                                                    <td class="px-4 py-2 border">
                                                        {card.last_reviewed.format("%Y-%m-%d %H:%M").to_string()}
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
