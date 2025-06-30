use leptos::prelude::*;

#[component]
pub fn Overview() -> impl IntoView {
    view! {
        <div class="mb-4 mt-12 text-center">
            <h2 class="font-styrene-display text-text-200 mb-1 text-2xl font-medium drop-shadow-sm md:text-3xl">
                <div style="opacity: 1; transform: none;">Learn Spanish</div>
            </h2>
        </div>

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

        <div class="flex flex-row gap-4 p-4">
            <a 
                class="w-full h-16 bg-gray-100 hover:bg-gray-200 text-black font-semibold text-lg rounded-xl transition-colors duration-200 shadow-lg hover:shadow-xl flex items-center justify-center"
                href="/learn-languages/write-story"
            >
                Write a story
            </a>
        </div>
    }
}