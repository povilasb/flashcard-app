#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::http::header::{CACHE_CONTROL, EXPIRES, PRAGMA};
    use axum::http::HeaderValue;
    use axum::response::Response;
    use axum::routing::{get, post};
    use axum::Router;
    use colog;
    use flashcard_app::api::cards;
    use flashcard_app::api::languages;
    use flashcard_app::app::*;
    use flashcard_app::settings::Settings;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use log::info;
    use tower_http::services::ServeDir;

    colog::init();
    Settings::load().expect("Failed to load settings");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .nest("/api/languages", Router::new()
            .route("/vocabulary", get(languages::vocabulary).post(languages::add_word).put(languages::update_word).delete(languages::delete_word))
            .route("/vocabulary/from-flashcards", post(languages::import))
            .route("/sentence", post(languages::sentence))
            .route("/story", post(languages::story))
            .route("/translation", get(languages::lookup))
            .route("/translate", post(languages::translate))
            .layer(axum::extract::DefaultBodyLimit::max(64 * 1024)))
        .route("/api/media", post(flashcard_app::api::media::upload)
            .layer(axum::extract::DefaultBodyLimit::max(flashcard_app::api::media::MAX_UPLOAD_BYTES + 64 * 1024)))
        .route("/api/rich-text/render", post(flashcard_app::api::rich_text::render))
        .route("/api/cards", post(cards::create_card).get(cards::get_all_cards))
        .route("/api/cards/review", get(cards::get_cards_to_review))
        .route("/api/cards/{id}/review", post(cards::submit_answer))
        .route("/api/cards/{id}", get(cards::get_card).put(cards::update_card).delete(cards::delete_card))
        .nest_service("/media", ServeDir::new(flashcard_app::api::media::MEDIA_DIR))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .layer(axum::middleware::map_response(
            |mut response: Response| async move {
                response.headers_mut().insert(
                    CACHE_CONTROL,
                    HeaderValue::from_static("no-cache, no-store, must-revalidate"),
                );
                response
                    .headers_mut()
                    .insert(PRAGMA, HeaderValue::from_static("no-cache"));
                response
                    .headers_mut()
                    .insert(EXPIRES, HeaderValue::from_static("0"));
                response
            },
        ));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
