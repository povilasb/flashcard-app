use leptos::prelude::*;
use leptos::task::spawn_local;

#[cfg(feature = "ssr")]
use crate::languages::ai;
#[cfg(feature = "ssr")]
use crate::languages::db::Database;
use crate::errors::AppError;

static LANG: &str = "spanish";

#[server(WriteStory, "/api")]
async fn write_story() -> Result<String, AppError> {
    ai::Agent::new("spanish").gen_story().await
}

#[server(GetTranslation, "/api")]
async fn get_translation(word: String) -> Result<Option<String>, AppError> {
    Ok(Database::get_instance(LANG).unwrap().lock().unwrap().get_translation(&word)?)
}

/// Writes a simple story using the words in my vocabulary.
#[component]
pub fn WriteStory() -> impl IntoView {
    let (story, set_story) = signal(None);
    let (hovered_word, set_hovered_word) = signal(None::<String>);
    let (translation, set_translation) = signal(None::<String>);
    let (tooltip_pos, set_tooltip_pos) = signal((0.0, 0.0));

    Effect::new(move || {
        spawn_local(async move {
            match write_story().await {
                Ok(s) => set_story.set(Some(s)),
                Err(e) => web_sys::console::error_1(&format!("Error writing story: {}", e).into()),
            }
        });
    });

    // Fetch translation when word changes
    Effect::new(move |_| {
        if let Some(word) = hovered_word.get() {
            let clean_word = word.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase();
            if !clean_word.is_empty() {
                spawn_local(async move {
                    match get_translation(clean_word).await {
                        Ok(Some(trans)) => set_translation.set(Some(trans)),
                        Ok(None) => set_translation.set(None),
                        Err(e) => web_sys::console::error_1(&format!("Error getting translation: {}", e).into()),
                    }
                });
            }
        } else {
            set_translation.set(None);
        }
    });

    view! {
        <div class="flex flex-col items-center h-screen">
            <h1 class="text-2xl font-bold">Story of the day</h1>
            <div class="mt-4 relative">
                {move || story.get()
                    .unwrap_or_default()
                    .split("\n")
                    .map(|line| line.to_string())
                    .map(|line| {
                        view! {
                            <p>
                                {line.split_inclusive(" ")
                                    .map(|word| word.to_string())
                                    .map(|word| {
                                        let word2 = word.clone();
                                        view! {
                                            <span 
                                                class="hover:bg-gray-100 cursor-pointer px-0.5 rounded relative" 
                                                on:mouseenter=move |ev| {
                                                    // Get mouse position for tooltip
                                                    let rect = event_target::<web_sys::Element>(&ev).get_bounding_client_rect();
                                                    set_tooltip_pos.set((rect.left() + rect.width() / 2.0, rect.top() - (rect.height() * 1.5)));
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
                                    .collect::<Vec<_>>()
                                }
                            </p>
                        }
                    })
                    .collect::<Vec<_>>()
                }
                
                // Translation tooltip
                {move || translation.get().map(|trans| {
                    let (x, y) = tooltip_pos.get();
                    view! {
                        <div 
                            class="fixed bg-black text-white px-2 py-1 rounded text-sm z-50 pointer-events-none shadow-lg"
                             style=format!("left: {}px; top: {}px; transform: translateX(-50%);", x, y)
                        >
                            {trans}
                        </div>
                    }
                })}
            </div>
        </div>
    }
}