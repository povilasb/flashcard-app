# About

Flashcard app.
The less we forget.

## Usage

```sh
cargo leptos build
cargo run --bin=cli
```

## Why?

* Ability to reuse flashcards for other purposes, like exercising touch typing.
* Simple custom reviewing algorithms.
* Easy card sharing across devices with total data ownership and privacy.

## Caveats

* An image can be loaded only from `./db/media`. 

## Architecture

* Checkout the `src/app.rs` for different routes and views.