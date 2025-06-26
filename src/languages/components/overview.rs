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

#[component]
pub fn Overview() -> impl IntoView {
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
        <div class="flex flex-col gap-4">
            <h1 class="text-2xl font-bold">
                "Spanish"
            </h1>
            <br/ >
        </div>

            <div class="text-sm text-gray-500">
                "Total: " { move || words.get().len() }
            </div>

            <div class="overflow-x-auto">
                <table class="min-w-full bg-white border border-gray-300">
                    <thead>
                        <tr class="bg-gray-100">
                            <th class="px-4 py-2 border">"Word"</th>
                            <th class="px-4 py-2 border">"Translation"</th>
                            <th class="px-4 py-2 border">"Created at"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || words.get()
                            .into_iter()
                            .map(|word| {
                                view! {
                                    <tr class="hover:bg-gray-50">
                                        <td class="px-4 py-2 border">{word.word}</td>
                                        <td class="px-4 py-2 border">{word.translation}</td>
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