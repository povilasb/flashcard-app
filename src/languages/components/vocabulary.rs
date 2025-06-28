
use leptos::prelude::*;
use leptos::task::spawn_local;

#[cfg(feature = "ssr")]
use crate::languages::Database;
use crate::languages::Word;
use crate::components::error_notification::ErrorNotification;

static LANG: &str = "spanish";

#[server(GetWords, "/api")]
async fn get_words() -> Result<Vec<Word>, ServerFnError> {
    let db = Database::get_instance(LANG).unwrap().lock().unwrap();
    let words = db.all_words().map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(words)
}

#[server(UpdateWordTranslation, "/api")]
async fn update_word_translation(word: String, translation: String) -> Result<(), ServerFnError> {
    let db = Database::get_instance(LANG).unwrap().lock().unwrap();
    db.update_word_translation(&word, &translation).map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server(DeleteWord, "/api")]
async fn delete_word(word: String) -> Result<(), ServerFnError> {
    let db = Database::get_instance(LANG).unwrap().lock().unwrap();
    db.delete_word(&word).map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[component]
pub fn Vocabulary() -> impl IntoView {
    let (words, set_words) = signal(Vec::new());
    let (error, set_error) = signal(None::<String>);

    // Load words
    Effect::new(move |_| {
        spawn_local(async move {
            match get_words().await {
                Ok(loaded_words) => {
                    set_words.set(loaded_words);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load cards:\n {}", e)));
                }
            }
        });
    });

    view! {
        <div class="text-sm text-gray-500">
            "Total: " { move || words.get().len() }
        </div>

        <div class="overflow-x-auto">
            <table class="min-w-full bg-white border border-gray-300">
                <thead>
                    <tr class="bg-gray-100">
                        <th class="px-4 py-2 border">Actions</th>
                        <th class="px-4 py-2 border">"Word"</th>
                        <th class="px-4 py-2 border">"Translation"</th>
                        <th class="px-4 py-2 border">"Created at"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || words.get()
                        .into_iter()
                        .map(|word| {
                            let word_text = word.word.clone();
                            let word_text2 = word.word.clone();
                            view! {
                                <tr class="hover:bg-gray-50">
                                    <td class="px-4 py-2 border">
                                        <svg class="w-5 h-5 cursor-pointer hover:text-red-700" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" on:click=move |_| {
                                            if let Some(window) = web_sys::window() {
                                                if let Ok(confirmed) = window.confirm_with_message(&format!("Are you sure you want to delete '{}'?", word_text2)) {
                                                    if confirmed {
                                                        let word_to_delete = word_text2.clone();
                                                        let set_words_clone = set_words.clone();
                                                        let set_error_clone = set_error.clone();
                                                        spawn_local(async move {
                                                            match delete_word(word_to_delete).await {
                                                                Ok(_) => {
                                                                    // Refresh the words list
                                                                    match get_words().await {
                                                                        Ok(updated_words) => {
                                                                            set_words_clone.set(updated_words);
                                                                        }
                                                                        Err(e) => {
                                                                            set_error_clone.set(Some(format!("Failed to refresh words after deletion:\n {}", e)));
                                                                        }
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    set_error_clone.set(Some(format!("Failed to delete word:\n {}", e)));
                                                                }
                                                            }
                                                        });
                                                    }
                                                }
                                            }
                                        }>
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                                        </svg>
                                    </td>
                                    <td class="px-4 py-2 border">{word_text}</td>
                                    <td class="px-4 py-2 border">
                                        <input type="text" value=word.translation.unwrap_or_default() on:change=move |ev| {
                                            let value = event_target_value(&ev);
                                            let word_text = word.word.clone();
                                            spawn_local(async move {
                                                if let Err(e) = update_word_translation(word_text, value).await {
                                                    set_error.set(Some(format!("Failed to update word translation:\n {}", e)));
                                                }
                                            });
                                        } />
                                    </td>
                                    <td class="px-4 py-2 border">{word.created_at.to_string()}</td>
                                </tr>
                            }
                        })
                        .collect_view()
                    }
                </tbody>
            </table>
        </div>

        <ErrorNotification error=error />
    }
}