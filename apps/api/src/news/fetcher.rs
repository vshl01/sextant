//! Background fetcher.
//!
//! A single tokio task, spawned at startup, polls every RSS source on a
//! timer and upserts deduped, coin-tagged rows into `articles`. The HTTP
//! read path in `news::service` never calls into here — Postgres is the
//! contract between the two halves.

use std::time::Duration;

use anyhow::Result;
use futures::future::join_all;
use sqlx::PgPool;

use crate::news::service::Article;
use crate::news::sources::{SOURCES, Source};
use crate::news::tagger;

/// Spawn the news fetcher as a long-lived tokio task. The task runs once
/// immediately, then every `interval` after that.
pub fn spawn(db: PgPool, interval: Duration) {
    tokio::spawn(async move {
        // `tokio::time::interval` fires immediately on the first tick, which
        // is what we want — populate the DB at startup, then poll on schedule.
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            if let Err(err) = run_once(&db).await {
                tracing::warn!(error = ?err, "news fetcher tick failed");
            }
        }
    });
}

/// One full fan-out: hit every source in parallel, upsert every parsed entry.
/// Per-source failures are logged and skipped; one bad feed does not abort
/// the tick.
async fn run_once(db: &PgPool) -> Result<()> {
    let fetches = SOURCES.iter().map(fetch_source);
    let results = join_all(fetches).await;

    let mut total = 0usize;
    let mut inserted = 0usize;

    for r in results {
        match r {
            Ok(articles) => {
                for a in articles {
                    total += 1;
                    if upsert(db, &a).await? {
                        inserted += 1;
                    }
                }
            }
            Err(err) => {
                tracing::warn!(error = ?err, "feed fetch failed");
            }
        }
    }

    tracing::info!(total, inserted, "news fetch complete");
    Ok(())
}

/// Insert a single article. Returns `true` if a new row was written, `false`
/// if the link was already present (duplicate from a prior tick).
async fn upsert(db: &PgPool, a: &Article) -> Result<bool> {
    let result = sqlx::query(
        "INSERT INTO articles (source, title, link, snippet, published, coins)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (link) DO NOTHING",
    )
    .bind(&a.source)
    .bind(&a.title)
    .bind(&a.link)
    .bind(&a.snippet)
    .bind(a.published)
    .bind(&a.coins)
    .execute(db)
    .await?;

    Ok(result.rows_affected() == 1)
}

/// Fetch and parse a single RSS source, tagging coins as we go so the
/// resulting `Article` values are ready to upsert.
async fn fetch_source(source: &Source) -> Result<Vec<Article>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("Mozilla/5.0 (compatible; CryptoBackend/0.1)")
        .build()?;
    let body = client.get(source.url).send().await?.bytes().await?;
    let feed = feed_rs::parser::parse(body.as_ref())?;

    let articles = feed
        .entries
        .into_iter()
        .map(|e| {
            let title = e.title.map(|t| t.content).unwrap_or_default();
            let snippet = e.summary.map(|s| s.content).unwrap_or_default();
            let coins = tagger::tag_coins(&title, &snippet);
            Article {
                source: source.name.to_string(),
                title,
                link: e.links.first().map(|l| l.href.clone()).unwrap_or_default(),
                snippet,
                published: e.published.or(e.updated),
                coins,
            }
        })
        .collect();

    Ok(articles)
}
