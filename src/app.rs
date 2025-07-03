use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    path, StaticSegment,
};
use thaw::ssr::SSRMountStyleProvider;
use thaw::ConfigProvider;

use crate::components::add_card::AddCard;
use crate::components::edit_card::EditCard;
use crate::components::list_cards::ListCards;
use crate::components::review_by_tag::ReviewByTag;
use crate::components::review_cards::ReviewAllCards;
use crate::components::view_card::ViewCard;
use crate::languages::components::{GenerateSentence, Overview, Vocabulary, WriteStory};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <SSRMountStyleProvider>
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="utf-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1" />
                    <AutoReload options=options.clone() />
                    <HydrationScripts options />
                    <MetaTags />
                </head>
                <body>
                    <App />
                </body>
            </html>
        </SSRMountStyleProvider>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <ConfigProvider>
            // injects a stylesheet into the document <head>
            // id=leptos means cargo-leptos will hot-reload this stylesheet
            <Stylesheet id="leptos" href="/pkg/flashcard-app.css" />
            <Title text="Review flashcards" />
            <AppRouter />
        </ConfigProvider>
    }
}

/// A navbar with the view routes.
#[component]
fn AppRouter() -> impl IntoView {
    view! {
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
                        <A href="/learn-languages">
                            <div class="block px-4 py-2 text-gray-700 hover:bg-gray-200 rounded">
                                "Learn languages"
                            </div>
                        </A>
                    </div>
                </nav>
                <main class="flex-1 p-4">
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=StaticSegment("") view=AddCard />
                        <Route path=path!("/add-card") view=AddCard />
                        <Route path=path!("/review-cards") view=ReviewAllCards />
                        <Route path=path!("/review-cards/:tag") view=ReviewByTag />
                        <Route path=path!("/list-cards") view=ListCards />
                        <Route path=path!("/cards/edit/:id") view=EditCard />
                        <Route path=path!("/cards/:id") view=ViewCard />
                        <Route path=path!("/learn-languages") view=Overview />
                        <Route path=path!("/learn-languages/vocabulary") view=Vocabulary />
                        <Route
                            path=path!("/learn-languages/generate-sentence")
                            view=GenerateSentence
                        />
                        <Route path=path!("/learn-languages/write-story") view=WriteStory />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}
