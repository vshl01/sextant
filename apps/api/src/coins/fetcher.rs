//! Background fetcher: pulls coin metadata + 24h-window stats from CoinGecko
//! and upserts into `coins`. Runs every few hours — this is metadata, it
//! doesn't move minute-by-minute.

use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;
use sqlx::PgPool;

use crate::coins::source::{COINS, Source};

/// One entry from CoinGecko's /coins/markets response. We only deserialize
/// the fields we actually use; everything else in the JSON is ignored.
#[derive(Debug, Deserialize)]
struct Market {
    id: String,
    image: Option<String>,
    market_cap: Option<f64>,
    circulating_supply: Option<f64>,
    #[serde(rename = "price_change_percentage_1h_in_currency")]
    change_1h: Option<f64>,
    #[serde(rename = "price_change_percentage_24h_in_currency")]
    change_24h: Option<f64>,
    #[serde(rename = "price_change_percentage_7d_in_currency")]
    change_7d: Option<f64>,
}

pub fn spawn(db: PgPool, interval: Duration) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            ticker.tick().await;
            if let Err(err) = run_once(&db).await {
                tracing::warn!(error = ?err, "coins fetcher tick failed");
            }
        }
    });
}

async fn run_once(db: &PgPool) -> Result<()> {
    let ids: Vec<&str> = COINS.iter().map(|c| c.coingecko_id).collect();
    let url = format!(
        "https://api.coingecko.com/api/v3/coins/markets?vs_currency=usd&ids={}&price_change_percentage=1h,24h,7d",
        ids.join(",")
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (compatible; CryptoBackend/0.1)")
        .build()?;

    let markets: Vec<Market> = client.get(&url).send().await?.json().await?;

    let mut written = 0usize;
    for src in COINS {
        let market = markets.iter().find(|m| m.id == src.coingecko_id);
        if upsert(db, src, market).await? {
            written += 1;
        }
    }
    tracing::info!(written, "coins fetch complete");
    Ok(())
}

async fn upsert(db: &PgPool, src: &Source, market: Option<&Market>) -> Result<bool> {
    let icon_url = market.and_then(|m| m.image.clone());
    let market_cap = market.and_then(|m| m.market_cap);
    let circulating = market.and_then(|m| m.circulating_supply);
    let change_1h = market.and_then(|m| m.change_1h);
    let change_24h = market.and_then(|m| m.change_24h);
    let change_7d = market.and_then(|m| m.change_7d);

    let result = sqlx::query(
        "INSERT INTO coins (
             symbol, coingecko_id, binance_pair, name, icon_url,
             market_cap_usd, circulating,
             change_1h_pct, change_24h_pct, change_7d_pct, refreshed_at
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
         ON CONFLICT (symbol) DO UPDATE SET
             icon_url       = EXCLUDED.icon_url,
             market_cap_usd = EXCLUDED.market_cap_usd,
             circulating    = EXCLUDED.circulating,
             change_1h_pct  = EXCLUDED.change_1h_pct,
             change_24h_pct = EXCLUDED.change_24h_pct,
             change_7d_pct  = EXCLUDED.change_7d_pct,
             refreshed_at   = NOW()",
    )
    .bind(src.symbol)
    .bind(src.coingecko_id)
    .bind(src.binance_pair)
    .bind(src.name)
    .bind(&icon_url)
    .bind(market_cap)
    .bind(circulating)
    .bind(change_1h)
    .bind(change_24h)
    .bind(change_7d)
    .execute(db)
    .await?;

    Ok(result.rows_affected() == 1)
}
