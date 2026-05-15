//! HTTP handlers for the news feature.

use axum::{Json, extract::Query};
use serde::Deserialize;

use crate::news::service::{self, Article};
// use super::service::{self, Article};
#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
}

pub async fn search_news(Query(params): Query<SearchParams>) -> Json<Vec<Article>> {
    let articles = service::search(&params.q).await;
    Json(articles)
}
