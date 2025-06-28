use leptos::prelude::*;
use leptos::task::spawn_local;

#[cfg(feature = "ssr")]
use crate::languages::ai::gen_new_sentence;
use crate::languages::NewSentence;

static LANG: &str = "spanish";

#[server(GenerateSentence, "/api")]
async fn generate_sentence() -> Result<NewSentence, ServerFnError> {
    let sentence = gen_new_sentence(LANG).await.map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(sentence)
}

/// Using LLMs, generate a new sentence with a new word and its translation for iterative language learning.
#[component]
pub fn GenerateSentence() -> impl IntoView {
    let new_sentence = RwSignal::new(None);

    view! {
        <div class="mb-4 mt-12 text-center">
            <h2 class="font-styrene-display text-text-200 mb-1 text-2xl font-medium drop-shadow-sm md:text-3xl">
                Learn new words
            </h2>
        </div>
        <button class="bg-blue-500 text-white px-4 py-2 rounded-md cursor-pointer" on:click=move |_| {
            spawn_local(async move {
                match generate_sentence().await {
                    Ok(sentence) => new_sentence.set(Some(sentence)),
                    Err(e) => {
                        web_sys::console::error_1(&format!("Error generating sentence: {}", e).into());
                    }
                }
            });
        }>
            Generate Sentence
        </button>

        <div class="mt-4">
            <div class="font-bold">New Sentence</div>
            <p>{move || new_sentence.get().map(|s| s.text)}</p>
            <div class="font-bold">New Word</div>
            <p>{move || new_sentence.get().map(|s| s.new_word)}</p>
            <div class="font-bold">Translation</div>
            <p>{move || new_sentence.get().map(|s| s.translation)}</p>
        </div>
    }
}
