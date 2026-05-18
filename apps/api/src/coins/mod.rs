use axum::{Router, routing::get};

use crate::state::AppState;

pub mod fetcher;
pub mod handler;
pub mod service;
pub mod source;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/coins", get(handler::list))
        .route("/coins/{symbol}", get(handler::get))
}
