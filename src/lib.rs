//! Flashcard storage, language learning, rendering, and Axum HTTP routes.
pub mod api;
pub mod db;
mod errors;
pub mod languages;
mod model;
pub mod settings;

pub mod http_routes;
mod rich_text;
