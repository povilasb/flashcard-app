use leptos::*;
use leptos::prelude::*;
use gloo_timers::callback::Timeout;
#[cfg(feature = "ssr")]
use crate::db::Database;
use crate::model::Flashcard;
use leptos::wasm_bindgen::JsCast;


#[server(SubmitCard, "/api")]
pub async fn submit_card(
    question: String,
    answer: String,
    examples: String,
    source: Option<String>,
    tags: String,
    img_fname: Option<String>,
) -> Result<(), ServerFnError> {
    let db = Database::get_instance("flashcards.db").unwrap();
    let db = db.lock().unwrap();

    let mut card = Flashcard::new(question, answer);
    card.examples = Some(examples);
    card.source = source;
    card.tags = tags.split(',').map(|s| s.trim().to_string()).collect();
    card.img = img_fname;

    db.add_card(card).map_err(|e| ServerFnError::new(e.to_string()))
}

/// Reused to add or edit a card.
#[component]
pub fn FlashcardForm(
    #[prop(into)] card: Flashcard,
) -> impl IntoView {
    let img_fname = NodeRef::<html::Input>::new();

    view! {
        <div>
            <label class="flex flex-col gap-2">
                <span>Question*:</span>
                <input
                    class="border rounded px-3 py-2"
                    type="text"
                    name="question"
                    required=true
                    value=card.question
                />
            </label>
            <label class="flex flex-col gap-2">
                <span>Answer:</span>
                <textarea
                    class="border rounded px-3 py-2"
                    name="answer"
                    rows=4
                    cols=80
                >
                    {card.answer}
                </textarea>
            </label>
            <label class="flex flex-col gap-2">
                <span>Examples:</span>
                <textarea class="border rounded px-3 py-2" name="examples" rows=4 cols=80>
                    {card.examples}
                </textarea>
            </label>
            <label class="flex flex-col gap-2">
                <span>Source:</span>
                <input
                    class="border rounded px-3 py-2"
                    type="text"
                    name="source"
                    value=card.source
                />
            </label>
            <label class="flex flex-col gap-2">
                <span>Tags (comma separated):</span>
                <input
                    class="border rounded px-3 py-2"
                    type="text"
                    name="tags"
                    value=card.tags.join(",")
                />
            </label>
            <label class="flex flex-col gap-2">
                <span>Image:</span>
                <input
                    class="border rounded px-3 py-2"
                    type="file"
                    accept="image/*"
                    on:input=move |ev| {
                        if let Some(files) = ev.target().unwrap().unchecked_ref::<web_sys::HtmlInputElement>().files() {
                            let file_name = files.get(0).unwrap().name();
                            img_fname.get().unwrap().set_value(&file_name);
                        }
                    }
                />
                <input type="hidden" name="img_fname" node_ref=img_fname />
            </label>
        </div>
    }
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
                    <FlashcardForm card=Flashcard::new("".to_string(), "".to_string()) />
                    <button
                        class="bg-blue-500 text-white px-6 py-2 rounded hover:bg-blue-600 transition mt-4"
                        type="submit"
                    >
                        {"Create Flashcard"}
                    </button>
                </ActionForm>
                <Show when=move || show_ack.get()>
                    <div class="text-green-600 font-semibold mt-2">
                        {"Card added successfully!"}
                    </div>
                </Show>
            </div>
        </div>
    }
}
