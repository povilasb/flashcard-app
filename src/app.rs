use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    StaticSegment,
    path,
};
use crate::components::add_card::AddCard;
use crate::components::flashcard_deck::FlashcardDeck;
use crate::components::list_cards::ListCards;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/flashcard-app.css"/>

        <Title text="Review flashcards"/>

        <Router>
            <div class="flex min-h-screen">
                <nav class="w-64 bg-gray-100 p-4 border-r border-gray-200">
                    <div class="space-y-2">
                        <A href="/add-card">
                            <div class="block px-4 py-2 text-gray-700 hover:bg-gray-200 rounded">
                                "Add new card"
                            </div>
                        </A>
                        <A href="/review-cards">
                            <div class="block px-4 py-2 text-gray-700 hover:bg-gray-200 rounded">
                                "Review cards"
                            </div>
                        </A>
                        <A href="/list-cards">
                            <div class="block px-4 py-2 text-gray-700 hover:bg-gray-200 rounded">
                                "List cards"
                            </div>
                        </A>
                    </div>
                </nav>
                <main class="flex-1 p-4">
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=StaticSegment("") view=AddCard/>
                        <Route path=path!("/add-card") view=AddCard/>
                        <Route path=path!("/review-cards") view=FlashcardDeck/>
                        <Route path=path!("/list-cards") view=ListCards/>
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
