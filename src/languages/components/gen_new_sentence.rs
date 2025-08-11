use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::errors::AppError;
#[cfg(feature = "ssr")]
use crate::languages::ai;
use crate::languages::model::NewSentence;
use crate::settings::Language;

#[server(GenerateSentence, "/api")]
async fn generate_sentence() -> Result<(NewSentence, Language), AppError> {
    let agent = ai::Agent::from_settings();
    let sentence = agent.gen_new_sentence().await?;
    Ok((sentence, agent.lang))
}

/// Using LLMs, generate a new sentence with a new word and its translation for iterative language learning.
#[component]
pub fn GenerateSentence() -> impl IntoView {
    let new_sentence = RwSignal::new(None);
    let flashcard_tag = RwSignal::new("".to_string());

    view! {
        <div class="mb-4 mt-12 text-center">
            <h2 class="font-styrene-display text-text-200 mb-1 text-2xl font-medium drop-shadow-sm md:text-3xl">
                Learn new words
            </h2>
        </div>
        <button
            class="bg-blue-500 text-white px-4 py-2 rounded-md cursor-pointer"
            on:click=move |_| {
                spawn_local(async move {
                    match generate_sentence().await {
                        Ok((sentence, language)) => {
                            new_sentence.set(Some(sentence));
                            flashcard_tag.set(language.as_str().to_string());
                        }
                        Err(e) => {
                            web_sys::console::error_1(
                                &format!("Error generating sentence: {}", e).into(),
                            );
                        }
                    }
                });
            }
        >
            Generate Sentence
        </button>

        <div class="mt-4">
            <table class="w-full border-collapse border border-gray-300 rounded-lg overflow-hidden shadow-sm">
                <thead class="bg-gray-50">
                    <tr>
                        <th class="px-4 py-3 text-left font-semibold text-gray-700 border-b border-gray-300">
                            New Sentence
                        </th>
                        <th class="px-4 py-3 text-left font-semibold text-gray-700 border-b border-gray-300">
                            New Word
                        </th>
                        <th class="px-4 py-3 text-left font-semibold text-gray-700 border-b border-gray-300">
                            Translation
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <tr class="hover:bg-gray-50">
                        <td class="px-4 py-3 text-gray-900 border-b border-gray-200">
                            {move || new_sentence.get().map(|s| s.text)}
                        </td>
                        <td class="px-4 py-3 text-gray-900 border-b border-gray-200">
                            {move || new_sentence.get().map(|s| s.new_word)}
                        </td>
                        <td class="px-4 py-3 text-gray-900 border-b border-gray-200">
                            {move || new_sentence.get().map(|s| s.translation)}
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <div class="mt-4">
            <form action="/add-card">
                <input
                    type="hidden"
                    name="answer"
                    value=move || new_sentence.get().map(|s| s.text)
                />
                <input type="hidden" name="tag" value=move || flashcard_tag.get() />
                <input type="hidden" name="source" value="learning-languages app" />
                <button
                    type="submit"
                    class="bg-blue-500 text-white px-4 py-2 rounded-md cursor-pointer"
                >
                    Create flashcard
                </button>
            </form>
        </div>
    }
}
