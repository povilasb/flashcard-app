use leptos::*;
use leptos::prelude::*;
use gloo_timers::callback::Timeout;
#[cfg(feature = "ssr")]
use crate::db::Database;
use crate::model::Flashcard;


#[server(SubmitCard, "/api")]
pub async fn submit_card(
    question: String,
    answer: String,
    examples: String,
    source: Option<String>,
    tags: String,
) -> Result<(), ServerFnError> {
    let db = Database::get_instance("flashcards").unwrap();
    let mut db = db.lock().unwrap();

    let mut card = Flashcard::new(question, answer);
    card.examples = examples.lines().map(|s| s.to_string()).collect();
    card.source = source;
    card.tags = tags.split(',').map(|s| s.trim().to_string()).collect();

    db.add(card);
    db.save().unwrap();

    Ok(())
}

#[component]
pub fn AddCard() -> impl IntoView {
    let submit = ServerAction::<SubmitCard>::new();
    let form_ref = NodeRef::<leptos::html::Form>::new();
    let show_ack = RwSignal::new(false);

    // Watch for successful form submission.
    Effect::new(move |_| {
        if let Some(Ok(_)) = submit.value().get() {
            show_ack.set(true);
            if let Some(form) = form_ref.get() {
                form.reset();
            }
            Timeout::new(3000, move || show_ack.set(false)).forget();
        }
    });

    view! {
        <div class="max-w-[600px] mx-auto my-8 p-4">
            <div class="flex flex-col gap-4 w-full max-w-md bg-white p-8 rounded shadow">
                <ActionForm action=submit node_ref=form_ref>
                    <h2 class="text-2xl font-bold mb-4">{"Add a new card"}</h2>
                    <label class="flex flex-col gap-2">
                        <span>Question:</span>
                        <input class="border rounded px-3 py-2" type="text" name="question" required=true />
                    </label>
                    <label class="flex flex-col gap-2">
                        <span>Answer:</span>
                        <input class="border rounded px-3 py-2" type="text" name="answer" required=true />
                    </label>
                    <label class="flex flex-col gap-2">
                        <span>Examples (one per line):</span>
                        <textarea class="border rounded px-3 py-2" name="examples" rows=4 cols=80 />
                    </label>
                    <label class="flex flex-col gap-2">
                        <span>Source (Optional):</span>
                        <input class="border rounded px-3 py-2" type="text" name="source" />
                    </label>
                    <label class="flex flex-col gap-2">
                        <span>Tags (comma separated):</span>
                        <input class="border rounded px-3 py-2" type="text" name="tags" />
                    </label>
                    <button class="bg-blue-500 text-white px-6 py-2 rounded hover:bg-blue-600 transition mt-4" type="submit">{"Create Flashcard"}</button>
                </ActionForm>
                <Show when=move || show_ack.get()>
                    <div class="text-green-600 font-semibold mt-2">{"Card added successfully!"}</div>
                </Show>
            </div>
        </div>
    }
}
