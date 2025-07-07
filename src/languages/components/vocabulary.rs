use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::Pagination;

use crate::components::error_notification::ErrorNotification;
use crate::errors::AppError;
#[cfg(feature = "ssr")]
use crate::languages::ai;
use crate::languages::model::Word;
#[cfg(feature = "ssr")]
use crate::words_db;

static LANG: &str = "spanish";

#[server(GetWords, "/api")]
async fn get_words() -> Result<Vec<Word>, AppError> {
    Ok(words_db!(LANG).all_words()?)
}

#[server(UpdateWordTranslation, "/api")]
async fn update_word_translation(word: String, translation: String) -> Result<(), AppError> {
    words_db!(LANG).update_word_translation(&word, &translation)?;
    Ok(())
}

#[server(DeleteWord, "/api")]
async fn delete_word(word: String) -> Result<(), AppError> {
    words_db!(LANG).delete_word(&word)?;
    Ok(())
}

#[server(AddWord, "/api")]
async fn add_word(word: String, translation: String) -> Result<(), AppError> {
    words_db!(LANG).add_word(&word, &translation)?;
    Ok(())
}

#[server(AddFromFlashcards, "/api")]
async fn add_from_flashcards() -> Result<(), AppError> {
    ai::Agent::new(LANG).populate_words_db().await
}

fn refresh_words(set_words: WriteSignal<Vec<Word>>, set_error: WriteSignal<Option<String>>) {
    spawn_local(async move {
        match get_words().await {
            Ok(updated_words) => {
                set_words.set(updated_words);
            }
            Err(e) => {
                set_error.set(Some(format!("Failed to load cards:\n {}", e)));
            }
        }
    });
}

#[component]
pub fn Vocabulary() -> impl IntoView {
    let (words, set_words) = signal(Vec::new());
    let (error, set_error) = signal(None::<String>);
    let page = RwSignal::new(1);
    let page_count = Memo::new(move |_| (words.get().len() as f64 / 10.0).ceil() as usize);

    // Load words
    Effect::new(move |_| {
        refresh_words(set_words, set_error);
    });

    let submit_word_form = ServerAction::<AddWord>::new();
    // Handle form submission result
    Effect::new(move |_| {
        if let Some(result) = submit_word_form.value().get() {
            match result {
                Ok(_) => refresh_words(set_words, set_error),
                Err(e) => {
                    set_error.set(Some(format!("Failed to add word:\n {}", e)));
                }
            }
        }
    });

    view! {
        <div class="text-sm text-gray-500">"Total: " {move || words.get().len()}</div>

        <ActionForm action=submit_word_form>
            <input
                name="word"
                type="text"
                placeholder="Word"
                class="border rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <input
                name="translation"
                type="text"
                placeholder="Translation"
                class="border rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
            <button
                type="submit"
                class="bg-transparent hover:bg-gray-100 text-gray-600 hover:text-gray-800 p-2 rounded"
            >
                <svg
                    class="w-5 h-5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M12 4v16m8-8H4"
                    ></path>
                </svg>
            </button>
        </ActionForm>

        <div class="overflow-x-auto">
            <WordsTable words=words set_words=set_words set_error=set_error page=page />
            <div class="mt-2 flex justify-between">
                <Pagination page_count=page_count page=page />
                <button
                    class="bg-blue-500 hover:bg-blue-600 text-white px-4 py-2 rounded"
                    on:click=move |_| {
                        spawn_local(async move {
                            match add_from_flashcards().await {
                                Ok(_) => refresh_words(set_words, set_error),
                                Err(e) => {
                                    set_error
                                        .set(
                                            Some(format!("Failed to add from flashcards:\n {}", e)),
                                        );
                                }
                            }
                        });
                    }
                >
                    Add from flashcards
                </button>
            </div>
        </div>

        <ErrorNotification error=error />
    }
}

#[component]
fn WordsTable(
    #[prop(into)] words: ReadSignal<Vec<Word>>,
    #[prop(into)] set_words: WriteSignal<Vec<Word>>,
    #[prop(into)] set_error: WriteSignal<Option<String>>,
    #[prop(into)] page: Signal<usize>,
) -> impl IntoView {
    view! {
        <table class="min-w-full bg-white border border-gray-300">
            <thead>
                <tr class="bg-gray-100">
                    <th class="px-4 py-2 border"></th>
                    <th class="px-4 py-2 border">"Word"</th>
                    <th class="px-4 py-2 border">"Translation"</th>
                    <th class="px-4 py-2 border">"Created at"</th>
                </tr>
            </thead>
            <tbody>
                {move || {
                    let page_words = words
                        .get()
                        .into_iter()
                        .skip((page.get() - 1) * 10)
                        .take(10)
                        .collect::<Vec<_>>();
                    page_words
                        .into_iter()
                        .map(|word| {
                            let word_text = word.word.clone();
                            let word_text2 = word.word.clone();
                            view! {
                                <tr class="hover:bg-gray-50">
                                    <td class="px-4 py-2 border">
                                        <svg
                                            class="w-5 h-5 cursor-pointer hover:text-red-700"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                            xmlns="http://www.w3.org/2000/svg"
                                            on:click=move |_| {
                                                maybe_delete_word(word_text2.clone(), set_words, set_error)
                                            }
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                            ></path>
                                        </svg>
                                    </td>
                                    <td class="px-4 py-2 border">{word_text}</td>
                                    <td class="px-4 py-2 border">
                                        <input
                                            type="text"
                                            value=word.translation.unwrap_or_default()
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev);
                                                let word_text = word.word.clone();
                                                spawn_local(async move {
                                                    if let Err(e) = update_word_translation(word_text, value)
                                                        .await
                                                    {
                                                        set_error
                                                            .set(
                                                                Some(format!("Failed to update word translation:\n {}", e)),
                                                            );
                                                    }
                                                });
                                            }
                                        />
                                    </td>
                                    <td class="px-4 py-2 border">{word.created_at.to_string()}</td>
                                </tr>
                            }
                        })
                        .collect_view()
                }}
            </tbody>
        </table>
    }
}

fn maybe_delete_word(
    word_text: String,
    set_words: WriteSignal<Vec<Word>>,
    set_error: WriteSignal<Option<String>>,
) {
    if let Some(window) = web_sys::window() {
        if let Ok(confirmed) = window
            .confirm_with_message(&format!("Are you sure you want to delete '{}'?", word_text,))
        {
            if confirmed {
                let word_to_delete = word_text.clone();
                spawn_local(async move {
                    match delete_word(word_to_delete).await {
                        Ok(_) => refresh_words(set_words, set_error),
                        Err(e) => {
                            set_error.set(Some(format!("Failed to delete word:\n {}", e)));
                        }
                    }
                });
            }
        }
    }
}
