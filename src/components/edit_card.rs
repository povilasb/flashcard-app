use leptos::*;
use leptos::prelude::*;
use leptos_router::params::Params;
use leptos_router::hooks::use_params;
use crate::model::Flashcard;
use crate::components::add_card::FlashcardForm;
#[cfg(feature = "ssr")]
use crate::db::Database;
use gloo_timers::callback::Timeout;
use leptos::task::spawn_local;

#[server(GetCard, "/api")]
async fn get_card(id: i64) -> Result<Flashcard, ServerFnError> {
    let db = Database::get_instance("flashcards.db").map_err(|e| ServerFnError::new(e.to_string()))?;
    let db = db.lock().unwrap();
    db.get_card(id).map_err(|e| ServerFnError::new(e.to_string()))
}

#[server(UpdateCard, "/api")]
async fn update_card(
    id: i64,
    question: String,
    answer: String,
    examples: String,
    source: Option<String>,
    tags: String,
) -> Result<(), ServerFnError> {
    let db = Database::get_instance("flashcards.db").map_err(|e| ServerFnError::new(e.to_string()))?;
    let db = db.lock().unwrap();

    let mut card = Flashcard::new(question, answer);
    card.id = id;
    card.examples = Some(examples);
    card.source = source;
    card.tags = tags.split(',')
        .map(|s| s.trim().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    db.update_card(&card).map_err(|e| ServerFnError::new(e.to_string()))
}

#[derive(Params, PartialEq, Clone)]
struct EditCardParams {
    id: Option<i64>,
}


#[component]
pub fn EditCard(
) -> impl IntoView {
    let (card, set_card) = signal(None::<Flashcard>);
    let submit = ServerAction::<UpdateCard>::new();
    let form_ref = NodeRef::<leptos::html::Form>::new();
    let show_ack = RwSignal::new(false);

    let params = use_params::<EditCardParams>();
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

    // Watch for successful form submission
    Effect::new(move |_| {
        if let Some(Ok(_)) = submit.value().get() {
            show_ack.set(true);
            Timeout::new(3000, move || show_ack.set(false)).forget();
        }
    });

    view! {
        <div class="max-w-[600px] mx-auto my-8 p-4">
            <div class="flex flex-col gap-4 w-full max-w-md bg-white p-8 rounded shadow">
                <Show
                    when=move || card.get().is_some()
                    fallback=|| view! { <div>"Loading..."</div> }
                >
                    {move || {
                        let card = card.get().unwrap();
                        view! {
                            <ActionForm action=submit node_ref=form_ref>
                                <h2 class="text-2xl font-bold mb-4">{"Edit card"}</h2>
                                <input type="hidden" name="id" value=card.id />
                                <FlashcardForm card=card />
                                <button class="bg-blue-500 text-white px-6 py-2 rounded hover:bg-blue-600 transition mt-4" type="submit">
                                    "Update Flashcard"
                                </button>
                            </ActionForm>
                            <Show when=move || show_ack.get()>
                                <div class="text-green-600 font-semibold mt-2">{"Card updated successfully!"}</div>
                            </Show>
                        }
                    }}
                </Show>
            </div>
        </div>
    }
}
