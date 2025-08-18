# About

Flashcard app.
The less we forget.

## Usage

```sh
brew install duckdb
cargo install cargo-leptos
cargo leptos watch
```

## Why?

* Ability to reuse flashcards for other purposes, like exercising touch typing.
* Simple custom reviewing algorithms.
* Easy card sharing across devices with total data ownership and privacy.

## Caveats

* An image can be loaded only from `./db/media`. 

## Architecture

* Checkout the `src/app.rs` for different routes and views.