use crate::components::markdown::Markdown;
use crate::model;
use crate::model::FlashcardAnswer;
use leptos::prelude::*;
use leptos::*;

#[component]
pub fn Flashcard(
    #[prop(into)] card: model::Flashcard,
    #[prop(into)] on_answer: Callback<FlashcardAnswer>,
) -> impl IntoView {
    let (show_answer, set_show_answer) = signal(false);
    let (show_examples, set_show_examples) = signal(false);

    let handle_answer = move |answer: FlashcardAnswer| {
        on_answer.run(answer);
        set_show_answer.set(false);
    };

    let img_src = card.img.clone().map(|s| format!("/media/{}", s));
    let question_img_src = card.question_img.clone().map(|s| format!("/media/{}", s));

    view! {
        <div class="max-w-[600px] mx-auto my-8 p-4">
            <div class="bg-white border border-slate-200 rounded-lg p-6 mb-4 shadow text-left relative">
                <div class="flex flex-col items-start gap-4 justify-start">
                    <p class="m-0 text-[1.1rem] leading-6 text-left">
                        <Markdown text=card.question.clone() />
                    </p>
                    <Show
                        when=move || card.question_img.clone().is_some()
                        fallback=move || view! {}
                    >
                        <div class="mt-4">
                            <img
                                src=question_img_src.clone().unwrap_or_default()
                                alt="Question image"
                                class="max-w-full h-auto rounded-md border border-slate-200"
                            />
                        </div>
                    </Show>
                </div>
                <div
                    class="flex flex-col items-start gap-4 justify-start"
                    style:display=move || if show_answer.get() { "flex" } else { "none" }
                >
                    <p class="m-0 text-[1.1rem] leading-6 text-left">
                        <hr class="my-2 mb-4" />
                        <Markdown text=card.answer.clone() />
                        <Show when=move || card.img.clone().is_some() fallback=move || view! {}>
                            <div class="mt-4">
                                <img
                                    src=img_src.clone().unwrap_or_default()
                                    alt="Flashcard image"
                                    class="max-w-full h-auto rounded-md border border-slate-200"
                                />
                            </div>
                        </Show>
                    </p>
                </div>
                <Show when=move || show_examples.get() fallback=move || view! {}>
                    <div class="mt-4">
                        <div class="bg-gray-50 p-4 rounded-md">
                            <b>"Examples:"</b>
                            <br />
                            <Markdown text=card.examples.clone().unwrap_or_default() />
                        </div>
                    </div>
                </Show>

                <div>
                    <For
                        each=move || card.tags.clone()
                        key=|tag| tag.clone()
                        children=move |tag| {
                            view! {
                                <div class="inline-block bg-slate-100 text-slate-500 px-3 py-1 rounded-full text-sm mt-4 border border-slate-200 text-center">
                                    {tag}
                                </div>
                            }
                        }
                    />
                </div>
                <a
                    class="absolute bottom-4 right-12 text-slate-400 hover:text-slate-600 transition-colors"
                    title="Show examples"
                    href="#"
                    on:click=move |ev| {
                        ev.prevent_default();
                        set_show_examples.set(true);
                    }
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <circle cx="12" cy="12" r="10" />
                        <path d="M8 12h8M12 8v8" />
                    </svg>
                </a>
                <div class="absolute bottom-4 right-20 text-slate-400 text-xs">
                    "Last reviewed: "{card.last_reviewed.format("%Y-%m-%d %H:%M").to_string()}
                </div>
                <a
                    class="absolute bottom-4 right-4 text-slate-400 hover:text-slate-600 transition-colors"
                    title="Edit card"
                    href=format!("/cards/edit/{}", card.id.to_string())
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                    >
                        <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                    </svg>
                </a>
            </div>
            <div class="mt-4 flex justify-center">
                <Show
                    when=move || !show_answer.get()
                    fallback=move || {
                        view! {
                            <div class="flex gap-4">
                                <button
                                    class="bg-blue-100 text-blue-700 border-none rounded-md px-6 py-3 text-base cursor-pointer transition-colors hover:bg-blue-200"
                                    on:click=move |_| handle_answer(FlashcardAnswer::Remember)
                                >
                                    "Remember"
                                </button>
                                <button
                                    class="bg-red-100 text-red-600 border-none rounded-md px-6 py-3 text-base cursor-pointer transition-colors hover:bg-red-200"
                                    on:click=move |_| handle_answer(FlashcardAnswer::Not)
                                >
                                    "Not"
                                </button>
                            </div>
                        }
                    }
                >
                    <div class="flex gap-4">
                        <button
                            on:click=move |_| set_show_answer.update(|value| *value = !*value)
                            class="bg-blue-100 text-blue-700 border-none rounded-md px-6 py-3 text-base cursor-pointer transition-colors hover:bg-blue-200"
                        >
                            "Show Answer"
                        </button>
                    </div>
                </Show>
            </div>
        </div>
    }
}
