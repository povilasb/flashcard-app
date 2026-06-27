#[cfg(feature = "ssr")]
pub mod cards;
pub mod client;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCardRequest {
    pub question: String,
    pub answer: String,
    pub examples: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub answer_img_fname: Option<String>,
    pub question_img_fname: Option<String>,
}
