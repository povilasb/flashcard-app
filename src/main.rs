//! Run the Axum API and serve the built Svelte frontend.
use flashcard_app::{http_routes, settings::Settings};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    colog::init();
    Settings::load()?;

    let packaged = std::env::current_exe()?.parent().unwrap().join("frontend");
    let frontend = if packaged.join("index.html").is_file() {
        packaged
    } else {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("svelte-frontend/dist")
    };
    if !frontend.join("index.html").is_file() {
        anyhow::bail!("Svelte build missing. Run bun run build in svelte-frontend first.");
    }

    let listener = tokio::net::TcpListener::bind("127.0.0.1:7001").await?;
    log::info!("Listening on http://127.0.0.1:7001");
    axum::serve(listener, http_routes::setup(frontend)).await?;
    Ok(())
}
