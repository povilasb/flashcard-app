# About

The less we forget.
Flashcard app with local only data storage.

## Run

```sh
cd svelte-frontend
bun install --frozen-lockfile
bun run build
cd ..
cargo run --locked
```

Open http://127.0.0.1:7001. Axum serves the API, media, and built frontend, including direct page links.
For frontend development, run `bun run dev` in `svelte-frontend` alongside Rust; Vite proxies `/api` and `/media` to port 7001.

Settings are read from `./settings.toml` when present, otherwise the OS config directory's `flashcard-app/settings.toml`.
The same `db_path`, learning language, provider credentials, and SQLite files are retained. Media remains in `./db/media`.
No data migration is needed. Back up your databases before upgrading; run only one server against your data.

## Packaging

`./build.sh` creates `flashcard-app.tar.gz` with the Rust binary and its sibling `frontend/` assets.
Extract and run the binary from any directory; frontend paths resolve beside the executable. Settings/database paths follow the existing rules above.
No Bun, Node, browser WASM, or frontend framework runtime server is needed in the installed package.