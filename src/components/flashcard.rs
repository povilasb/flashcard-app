use leptos::*;
use leptos::prelude::*;
use crate::model::FlashcardAnswer;
use crate::model;

#[component]
pub fn Flashcard(
    #[prop(into)] card: model::Flashcard,
    #[prop(into)] on_answer: Callback<FlashcardAnswer>,
) -> impl IntoView {
    let (show_answer, set_show_answer) = signal(false);

    let handle_answer = move |answer: FlashcardAnswer| {
        on_answer.run(answer);
        set_show_answer.set(false);
    };

    view! {
        <div class="max-w-[600px] mx-auto my-8 p-4">
            <div class="bg-white border border-slate-200 rounded-lg p-6 mb-4 shadow text-left relative">
                <div class="flex items-start gap-4 justify-start">
                    <p class="m-0 text-[1.1rem] leading-6 text-left"><b>"Q: "</b>{card.question}</p>
                </div>
                <div class="flex items-start gap-4 justify-start" style:display=move || if show_answer.get() { "flex" } else { "none" }>
                    <p class="m-0 text-[1.1rem] leading-6 text-left" style="white-space: pre-wrap"><b>"A: "</b><br/>{card.answer}</p>
                </div>
                <div>
                    <For
                        each=move || card.tags.clone()
                        key=|tag| tag.clone()
                        children=move |tag| view! {
                            <div class="inline-block bg-slate-100 text-slate-500 px-3 py-1 rounded-full text-sm mt-4 border border-slate-200 text-center">
                                {tag}
                            </div>
                        }
                    />
                </div>
                <a
                    class="absolute bottom-4 right-4 text-slate-400 hover:text-slate-600 transition-colors"
                    title="Edit card"
                    href=format!("/cards/edit/{}", card.id.to_string())
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/>
                    </svg>
                </a>
            </div>
            <div class="mt-4 flex justify-center">
                <Show
                    when=move || !show_answer.get()
                    fallback=move || view! {
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