//! Flashcard storage, language learning, rendering, and Axum HTTP routes.
pub mod api;
pub mod db;
mod errors;
pub mod languages;
mod model;
pub mod settings;

mod rich_text;
pub mod server;
