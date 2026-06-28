use gloo_timers::callback::Timeout;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::*;
use leptos_router::{hooks::use_query, params::Params};

use crate::model::Flashcard;
use leptos::wasm_bindgen::JsCast;

#[cfg(feature = "hydrate")]
use crate::api::client::create_card;
#[cfg(feature = "hydrate")]
use crate::api::CreateCardRequest;

/// Reused to add or edit a card.
#[component]
pub fn FlashcardForm(#[prop(into)] card: Flashcard) -> impl IntoView {
    let answer_img_fname = NodeRef::<html::Input>::new();
    let question_img_fname = NodeRef::<html::Input>::new();

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
                <label class="flex flex-col gap-2 ml-4">
                    <span>Image:</span>
                    <input
                        class="border rounded px-3 py-2"
                        type="file"
                        accept="image/*"
                        on:input=move |ev| {
                            if let Some(files) = ev
                                .target()
                                .unwrap()
                                .unchecked_ref::<web_sys::HtmlInputElement>()
                                .files()
                            {
                                let file_name = files.get(0).unwrap().name();
                                question_img_fname.get().unwrap().set_value(&file_name);
                            }
                        }
                    />
                    <input type="hidden" name="question_img_fname" node_ref=question_img_fname />
                </label>
            </label>
            <label class="flex flex-col gap-2">
                <span>Answer:</span>
                <textarea class="border rounded px-3 py-2" name="answer" rows=4 cols=80>
                    {card.answer}
                </textarea>
                <label class="flex flex-col gap-2 ml-4">
                    <span>Image:</span>
                    <input
                        class="border rounded px-3 py-2"
                        type="file"
                        accept="image/*"
                        on:input=move |ev| {
                            if let Some(files) = ev
                                .target()
                                .unwrap()
                                .unchecked_ref::<web_sys::HtmlInputElement>()
                                .files()
                            {
                                let file_name = files.get(0).unwrap().name();
                                answer_img_fname.get().unwrap().set_value(&file_name);
                            }
                        }
                    />
                    <input type="hidden" name="answer_img_fname" node_ref=answer_img_fname />
                </label>
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
        </div>
    }
}

#[derive(Params, PartialEq, Clone)]
struct AddCardParams {
    question: Option<String>,
    answer: Option<String>,
    source: Option<String>,
    tag: Option<String>,
}

#[component]
pub fn AddCard() -> impl IntoView {
    let params = use_query::<AddCardParams>();
    let question = params
        .get()
        .map(|p| p.question)
        .unwrap_or_default()
        .unwrap_or_default();
    let answer = params
        .get()
        .map(|p| p.answer)
        .unwrap_or_default()
        .unwrap_or_default();
    let mut card = Flashcard::new(question, answer);
    if let Some(source) = params.get().map(|p| p.source).unwrap_or_default() {
        card.source = Some(source);
    }
    if let Some(tag) = params.get().map(|p| p.tag).unwrap_or_default() {
        card.tags = vec![tag];
    }

    let form_ref = NodeRef::<leptos::html::Form>::new();
    let show_ack = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(form) = form_ref.get() else { return };
        let form_el = form.unchecked_ref::<web_sys::HtmlFormElement>();
        let Ok(form_data) = web_sys::FormData::new_with_form(form_el) else { return };

        let get = |name: &str| form_data.get(name).as_string().unwrap_or_default();
        let get_opt = |name: &str| form_data.get(name).as_string().filter(|s| !s.is_empty());

        let tags_str = get("tags");
        let tags = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        #[cfg(feature = "hydrate")]
        {
            let req = CreateCardRequest {
                question: get("question"),
                answer: get("answer"),
                examples: get_opt("examples"),
                source: get_opt("source"),
                tags,
                answer_img_fname: get_opt("answer_img_fname"),
                question_img_fname: get_opt("question_img_fname"),
            };

            spawn_local(async move {
                match create_card(&req).await {
                    Ok(_) => {
                        show_ack.set(true);
                        form.reset();
                        Timeout::new(3000, move || show_ack.set(false)).forget();
                    }
                    Err(e) => web_sys::console::error_1(&e.to_string().into()),
                }
            });
        }
    };

    view! {
        <div class="max-w-[600px] mx-auto my-8 p-4">
            <div class="flex flex-col gap-4 w-full max-w-md bg-white p-8 rounded shadow">
                <form node_ref=form_ref on:submit=on_submit>
                    <h2 class="text-2xl font-bold mb-4">{"Add a new card"}</h2>
                    <FlashcardForm card=card />
                    <button
                        class="bg-blue-500 text-white px-6 py-2 rounded hover:bg-blue-600 transition mt-4"
                        type="submit"
                    >
                        {"Create Flashcard"}
                    </button>
                </form>
                <Show when=move || show_ack.get()>
                    <div class="text-green-600 font-semibold mt-2">
                        {"Card added successfully!"}
                    </div>
                </Show>
            </div>
        </div>
    }
}
