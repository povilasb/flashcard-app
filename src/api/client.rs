//! API Web client.

#[cfg(not(feature = "ssr"))]
use gloo_net::http::Request;

use crate::{api::CreateCardRequest, model::Flashcard};

#[cfg(not(feature = "ssr"))]
pub async fn fetch_card(id: i64) -> Result<Flashcard, gloo_net::Error> {
    Request::get(&format!("/api/cards/{id}"))
        .send()
        .await?
        .json::<Flashcard>()
        .await
}

#[cfg(not(feature = "ssr"))]
pub async fn create_card(req: &CreateCardRequest) -> Result<(), gloo_net::Error> {
    Request::post("/api/cards")
        .json(req)?
        .send()
        .await
        .map(|_| ())
}
