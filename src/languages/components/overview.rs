use leptos::prelude::*;

#[component]
pub fn Overview() -> impl IntoView {
    view! {
        <div class="flex flex-row gap-4 p-4">
            <a 
                class="w-full h-16 bg-gray-100 hover:bg-gray-200 text-black font-semibold text-lg rounded-xl transition-colors duration-200 shadow-lg hover:shadow-xl flex items-center justify-center"
                href="/learn-languages/vocabulary"
            >
                Vocabulary
            </a>
            <a 
                class="w-full h-16 bg-gray-100 hover:bg-gray-200 text-black font-semibold text-lg rounded-xl transition-colors duration-200 shadow-lg hover:shadow-xl flex items-center justify-center"
                href="/learn-languages/generate-sentence"
            >
                Generate Sentence
            </a>
        </div>
    }
}