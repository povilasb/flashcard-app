#!/bin/bash
set -euo pipefail

bun install --frozen-lockfile --cwd svelte-frontend
(cd svelte-frontend && bun run build)
cargo build --locked --release --bin flashcard-app

package_dir=$(mktemp -d)
trap 'rm -r "$package_dir"' EXIT
mkdir -p "$package_dir/flashcard-app/frontend"
cp target/release/flashcard-app "$package_dir/flashcard-app/"
cp -R svelte-frontend/dist/. "$package_dir/flashcard-app/frontend/"
tar -czf flashcard-app.tar.gz -C "$package_dir" flashcard-app
