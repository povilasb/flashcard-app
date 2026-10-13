//! Route APIs and media separately from the Svelte document fallback.
use crate::api::{cards, languages, media, rich_text};
use axum::{
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use std::path::PathBuf;
use tower_http::services::ServeDir;

pub fn router(frontend: PathBuf) -> Router {
    let index = frontend.join("index.html");
    let document = Router::new().fallback(move |uri: Uri| {
        let index = index.clone();
        async move { frontend_document(uri, index).await }
    });
    Router::new()
        .nest(
            "/api/languages",
            Router::new()
                .route(
                    "/vocabulary",
                    get(languages::vocabulary)
                        .post(languages::add_word)
                        .put(languages::update_word)
                        .delete(languages::delete_word),
                )
                .route("/vocabulary/from-flashcards", post(languages::import))
                .route("/sentence", post(languages::sentence))
                .route("/story", post(languages::story))
                .route("/translation", get(languages::lookup))
                .route("/translate", post(languages::translate))
                .layer(axum::extract::DefaultBodyLimit::max(64 * 1024)),
        )
        .route(
            "/api/media",
            post(media::upload).layer(axum::extract::DefaultBodyLimit::max(
                media::MAX_UPLOAD_BYTES + 64 * 1024,
            )),
        )
        .route("/api/rich-text/render", post(rich_text::render))
        .route(
            "/api/cards",
            post(cards::create_card).get(cards::get_all_cards),
        )
        .route("/api/cards/review", get(cards::get_cards_to_review))
        .route("/api/cards/{id}/review", post(cards::submit_answer))
        .route(
            "/api/cards/{id}",
            get(cards::get_card)
                .put(cards::update_card)
                .delete(cards::delete_card),
        )
        .route("/api", get(|| async { StatusCode::NOT_FOUND }))
        .route("/api/{*path}", get(|| async { StatusCode::NOT_FOUND }))
        .nest_service("/media", ServeDir::new(media::MEDIA_DIR))
        .fallback_service(ServeDir::new(frontend).fallback(document))
        .layer(axum::middleware::from_fn(
            |request: axum::extract::Request, next: axum::middleware::Next| async move {
                let api = request.uri().path().starts_with("/api");
                let mut response = next.run(request).await;
                if api {
                    response
                        .headers_mut()
                        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
                } else if response
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .is_some_and(|value| value.as_bytes().starts_with(b"text/html"))
                {
                    response
                        .headers_mut()
                        .insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
                }
                response
            },
        ))
}

async fn frontend_document(uri: Uri, index: PathBuf) -> Response {
    if std::path::Path::new(uri.path()).extension().is_some()
        && !uri.path().starts_with("/review-cards/")
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    match tokio::fs::read(index).await {
        Ok(html) => (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            html,
        )
            .into_response(),
        Err(error) => {
            log::error!("Cannot read Svelte document: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn serves_deep_links_but_not_missing_apis_media_or_assets() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("index.html"), "<html>Svelte</html>").unwrap();
        let app = router(dir.path().to_path_buf());

        for path in [
            "/",
            "/cards/edit/1",
            "/learn-languages/vocabulary",
            "/review-cards/spanish",
            "/review-cards/chapter.3",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-cache");
            assert_eq!(
                to_bytes(response.into_body(), 1024).await.unwrap(),
                "<html>Svelte</html>"
            );
        }

        for path in [
            "/api/missing",
            "/api/languages/missing",
            "/media/missing.png",
            "/assets/missing.js",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{path}");
        }
    }
}
