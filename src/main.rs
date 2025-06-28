#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use flashcard_app::app::*;
    use axum::http::header::{CACHE_CONTROL, PRAGMA, EXPIRES};
    use axum::http::HeaderValue;
    use axum::response::Response;
    use tower_http::services::ServeDir;
    use dotenv::dotenv;

    // Load environment variables from .env file
    dotenv().ok();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .nest_service("/media", ServeDir::new("db/media"))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .layer(axum::middleware::map_response(|mut response: Response| async move {
            response.headers_mut().insert(CACHE_CONTROL, HeaderValue::from_static("no-cache, no-store, must-revalidate"));
            response.headers_mut().insert(PRAGMA, HeaderValue::from_static("no-cache"));
            response.headers_mut().insert(EXPIRES, HeaderValue::from_static("0"));
            response
        }));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
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
