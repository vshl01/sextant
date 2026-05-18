use axum::{Router, routing::get};

use crate::state::AppState;

pub mod fetcher;
pub mod handler;
pub mod service;
pub mod sources;
pub mod tagger;

pub fn router() -> Router<AppState> {
    Router::new().route("/news", get(handler::search_news))
}
