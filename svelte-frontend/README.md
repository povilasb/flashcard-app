# About

Web based frontend for flashcard app.

```sh
bun install
bun run dev
```

`bun run check` checks types. `bun run build` builds production assets into `dist/`.

`bun run format` formats files. `bun run format:check` checks formatting.

Run the Rust server on `127.0.0.1:7001`. Vite proxies `/api` to it during development.

Production: `bun run build`, then `cargo run --locked` from the repository root.
Axum serves `dist/` and all page routes; no Leptos or browser WASM is involved.
