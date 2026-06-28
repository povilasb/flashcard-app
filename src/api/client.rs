//! API Web client.

#[cfg(feature = "hydrate")]
use gloo_net::http::Request;

use crate::{api::CreateCardRequest, model::Flashcard};

#[cfg(feature = "hydrate")]
pub async fn fetch_card(id: i64) -> Result<Flashcard, gloo_net::Error> {
    Request::get(&format!("/api/cards/{id}"))
        .send()
        .await?
        .json::<Flashcard>()
        .await
}

#[cfg(feature = "hydrate")]
pub async fn create_card(req: &CreateCardRequest) -> Result<(), gloo_net::Error> {
    Request::post("/api/cards")
        .json(req)?
        .send()
        .await
        .map(|_| ())
}

#[cfg(feature = "hydrate")]
pub async fn delete_card(id: i64) -> Result<(), gloo_net::Error> {
    Request::delete(&format!("/api/cards/{id}"))
        .send()
        .await
        .map(|_| ())
}
