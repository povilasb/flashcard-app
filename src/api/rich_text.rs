//! Rich-text rendering for browser clients using the existing Rust renderer.

use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RenderRequest {
    pub text: String,
}

#[derive(Serialize)]
pub struct RenderResponse {
    pub html: String,
}

pub async fn render(Json(request): Json<RenderRequest>) -> Json<RenderResponse> {
    Json(RenderResponse {
        html: crate::components::render_rich_text(&request.text),
    })
}
