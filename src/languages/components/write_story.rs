use leptos::prelude::*;
use leptos::task::spawn_local;

#[cfg(feature = "ssr")]
use crate::languages::ai;
use crate::errors::AppError;

#[server(WriteStory, "/api")]
async fn write_story() -> Result<String, AppError> {
    ai::Agent::new("spanish").gen_story().await
}

/// Writes a simple story using the words in my vocabulary.
#[component]
pub fn WriteStory() -> impl IntoView {
    let (story, set_story) = signal(None);

    Effect::new(move || {
        spawn_local(async move {
            match write_story().await {
                Ok(s) => set_story.set(Some(s)),
                Err(e) => web_sys::console::error_1(&format!("Error writing story: {}", e).into()),
            }
        });
    });

    view! {
        <div class="flex flex-col items-center h-screen">
            <h1 class="text-2xl font-bold">Story of the day</h1>
            <div class="mt-4">
                {move || story.get()
                    .unwrap_or_default()
                    .split("\n")
                    .map(|line| line.to_string())
                    .map(|line| {
                        view! {
                            <p>
                                {line.split_inclusive(" ")
                                    .map(|word| word.to_string())
                                    .map(|word| view! {
                                        <span class="hover:bg-gray-100 cursor-pointer px-0.5 rounded">
                                            {word}
                                        </span>
                                    })
                                    .collect::<Vec<_>>()
                                }
                            </p>
                        }
                    })
                    .collect::<Vec<_>>()
                }
            </div>
        </div>
    }
}