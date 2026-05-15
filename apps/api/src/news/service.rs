use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::future::join_all;
use serde::Serialize;
use std::time::Duration;

use crate::news::sources::{SOURCES, Source};

#[derive(Clone, Serialize)]
pub struct Article {
    pub title: String,
    pub link: String,
    pub source: String,
    pub snippet: String,
    pub published: Option<DateTime<Utc>>,
}

pub async fn search(query: &str) -> Vec<Article> {
    let needle = query.to_lowercase();

    // Fan out: hit all sources in parallel.
    let fetches = SOURCES.iter().map(fetch_source);
    let results = join_all(fetches).await;

    // Flatten, drop failed sources, filter by query.
    let mut articles: Vec<Article> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .flatten()
        .filter(|a| {
            a.title.to_lowercase().contains(&needle) || a.snippet.to_lowercase().contains(&needle)
        })
        .collect();

    // Newest first.
    articles.sort_by(|a, b| b.published.cmp(&a.published));
    articles
}

async fn fetch_source(source: &Source) -> Result<Vec<Article>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .user_agent("Mozilla/5.0 (compatible; CryptoBackend/0.1)")
        .build()?;
    let body = client.get(source.url).send().await?.bytes().await?;
    let feed = feed_rs::parser::parse(body.as_ref())?;

    let articles = feed
        .entries
        .into_iter()
        .map(|e| Article {
            title: e.title.map(|t| t.content).unwrap_or_default(),
            link: e.links.first().map(|l| l.href.clone()).unwrap_or_default(),
            source: source.name.to_string(),
            snippet: e.summary.map(|s| s.content).unwrap_or_default(),
            published: e.published.or(e.updated),
        })
        .collect();

    Ok(articles)
}
