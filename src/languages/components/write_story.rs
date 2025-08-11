use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::Spinner;
use web_sys::window;

use crate::errors::AppError;
#[cfg(feature = "ssr")]
use crate::languages::ai;
#[cfg(feature = "ssr")]
use crate::settings::Settings;
#[cfg(feature = "ssr")]
use crate::words_db;
#[cfg(feature = "ssr")]
use translators::{GoogleTranslator, Translator};

#[server(WriteStory, "/api")]
async fn write_story() -> Result<(String, String), AppError> {
    let agent = ai::Agent::from_settings();
    let story = agent.gen_story().await?;
    Ok((story, agent.lang))
}

#[server(GetTranslation, "/api")]
async fn get_translation(word: String) -> Result<Option<String>, AppError> {
    Ok(words_db!().get_translation(&word)?)
}

#[server(Translators, "/api")]
async fn translators_translate(text: String) -> Result<Option<String>, AppError> {
    let settings = Settings::get();
    let target_lang = match settings.learning_language.as_str() {
        "spanish" => "es",
        "french" => "fr",
        "portuguese" => "pt-PT",
        _ => {
            return Err(AppError::GoogleTranslateError(format!(
                "Unsupported language: {}",
                settings.learning_language
            )))
        }
    };

    let google_trans = GoogleTranslator::default();
    let translation = google_trans
        .translate_async(&text, target_lang, "en")
        .await?;
    Ok(Some(translation))
}

/// Writes a simple story using the words in my vocabulary.
#[component]
pub fn WriteStory() -> impl IntoView {
    let story = OnceResource::new(async move { write_story().await.unwrap_or_default() });

    view! {
        <div class="flex flex-col h-screen">
            <h1 class="text-2xl font-bold text-center">Story of the day</h1>

            <Transition fallback=move || {
                view! { <Spinner /> }
            }>
                {move || Suspend::new(async move {
                    let (story, learning_language) = story.await;
                    view! { <Story story=story learning_language=learning_language /> }
                })}
            </Transition>
        </div>
    }
}

/// Story component that displays the story content with word hover functionality
#[component]
fn Story(
    #[prop(into)] story: Signal<String>,
    #[prop(into)] learning_language: String,
) -> impl IntoView {
    let (hovered_word, set_hovered_word) = signal(None::<String>);
    let (translation, set_translation) = signal(None::<String>);
    let (tooltip_pos, set_tooltip_pos) = signal((0.0, 0.0));
    let (selected_sentence, set_selected_sentence) = signal(None::<String>);
    let selected_translation = Resource::new(
        move || selected_sentence.get(),
        move |sentence| async move {
            if let Some(sentence) = sentence {
                translators_translate(sentence)
                    .await
                    .unwrap_or_default()
                    .unwrap_or_default()
            } else {
                "".to_string()
            }
        },
    );

    // Fetch translation when word changes
    Effect::new(move |_| {
        if let Some(word) = hovered_word.get() {
            let clean_word = word
                .trim_matches(|c: char| !c.is_alphabetic())
                .to_lowercase();
            if !clean_word.is_empty() {
                spawn_local(async move {
                    // First try to get translation from database
                    match get_translation(clean_word.clone()).await {
                        Ok(Some(trans)) => set_translation.set(Some(trans)),
                        Ok(None) => {
                            // If no translation in database, try translators
                            match translators_translate(clean_word).await {
                                Ok(Some(trans)) => set_translation.set(Some(trans)),
                                Ok(None) => set_translation.set(None),
                                Err(e) => web_sys::console::error_1(
                                    &format!("Error getting translators translation: {}", e).into(),
                                ),
                            }
                        }
                        Err(e) => web_sys::console::error_1(
                            &format!("Error getting translation: {}", e).into(),
                        ),
                    }
                });
            }
        } else {
            set_translation.set(None);
        }
    });

    // Function to get selected text
    let get_selected_text = move || {
        if let Some(window) = window() {
            if let Some(selection) = window.get_selection().ok().flatten() {
                let selected_text = String::from(selection.to_string()).trim().to_string();
                if !selected_text.is_empty() {
                    set_selected_sentence.set(Some(selected_text.to_string()));
                } else {
                    set_selected_sentence.set(None);
                }
            }
        }
    };

    // Function to clear selection
    let clear_selection = move || {
        if let Some(window) = window() {
            if let Some(selection) = window.get_selection().ok().flatten() {
                let _ = selection.remove_all_ranges();
                set_selected_sentence.set(None);
            }
        }
    };
    view! {
        <div
            class="mt-4 relative max-w-4xl mx-auto"
            on:mouseup=move |_| get_selected_text()
            on:mousedown=move |_| clear_selection()
        >
            {move || {
                let story_content = story.get();
                story_content
                    .split("\n")
                    .map(|line| line.to_string())
                    .map(|line| {
                        view! {
                            <p>
                                {line
                                    .split_inclusive(" ")
                                    .map(|word| word.to_string())
                                    .map(|word| {
                                        let word2 = word.clone();
                                        view! {
                                            <span
                                                class="hover:bg-gray-100 cursor-pointer px-0.5 rounded relative select-text"
                                                on:mouseenter=move |ev| {
                                                    let rect = event_target::<web_sys::Element>(&ev)
                                                        .get_bounding_client_rect();
                                                    set_tooltip_pos
                                                        .set((
                                                            rect.left() + rect.width() / 2.0,
                                                            rect.top() - (rect.height() * 1.5),
                                                        ));
                                                    set_hovered_word.set(Some(word2.clone()));
                                                }
                                                on:mouseleave=move |_| {
                                                    set_hovered_word.set(None);
                                                }
                                            >
                                                {word}
                                            </span>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            </p>
                        }
                    })
                    .collect::<Vec<_>>()
            }}

            // Translation tooltip
            {move || {
                translation
                    .get()
                    .map(|trans| {
                        let (x, y) = tooltip_pos.get();
                        view! {
                            <div
                                class="fixed bg-black text-white px-2 py-1 rounded text-sm z-50 pointer-events-none shadow-lg"
                                style=format!(
                                    "left: {}px; top: {}px; transform: translateX(-50%);",
                                    x,
                                    y,
                                )
                            >
                                {trans}
                            </div>
                        }
                    })
            }}
        </div>

        // Show selected sentence if any
        {move || {
            selected_sentence
                .get()
                .map(|sentence| {
                    let sentence2 = sentence.clone();
                    view! {
                        <div class="mt-2 p-2 bg-gray-100 rounded relative w-full">
                            <button
                                class="absolute top-1 right-1 hover:text-blue-800 text-sm cursor-pointer"
                                on:click=move |_| clear_selection()
                            >
                                X
                            </button>
                            <table class="w-full border-collapse">
                                <thead>
                                    <tr class="border-b">
                                        <th class="text-left p-2 font-semibold">Selected</th>
                                        <th class="text-left p-2 font-semibold">Translation</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <tr>
                                        <td class="p-2">{sentence}</td>
                                        <td class="p-2">{move || selected_translation.get()}</td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>

                        <div class="mt-4">
                            <form action="/add-card">
                                <input
                                    type="hidden"
                                    name="question"
                                    value=move || selected_translation.get()
                                />
                                <input type="hidden" name="answer" value=sentence2 />
                                <input type="hidden" name="tag" value=learning_language.clone() />
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
                })
        }}
    }
}
