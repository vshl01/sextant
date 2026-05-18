//! Read path for /coins. Queries the `coins` table that the background
//! fetcher in `coins::fetcher` keeps populated.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Coin {
    pub symbol: String,
    pub coingecko_id: String,
    pub binance_pair: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub market_cap_usd: Option<f64>,
    pub circulating: Option<f64>,
    pub change_1h_pct: Option<f64>,
    pub change_24h_pct: Option<f64>,
    pub change_7d_pct: Option<f64>,
    pub refreshed_at: DateTime<Utc>,
}

/// All tracked coins, biggest market cap first.
pub async fn list_all(db: &PgPool) -> Result<Vec<Coin>, AppError> {
    let rows = sqlx::query_as::<_, Coin>(
        "SELECT symbol, coingecko_id, binance_pair, name, icon_url,
                market_cap_usd, circulating,
                change_1h_pct, change_24h_pct, change_7d_pct,
                refreshed_at
         FROM coins
         ORDER BY market_cap_usd DESC NULLS LAST",
    )
    .fetch_all(db)
    .await?;
    Ok(rows)
}

/// One coin by symbol. `None` if it isn't tracked.
pub async fn get(db: &PgPool, symbol: &str) -> Result<Option<Coin>, AppError> {
    let row = sqlx::query_as::<_, Coin>(
        "SELECT symbol, coingecko_id, binance_pair, name, icon_url,
                market_cap_usd, circulating,
                change_1h_pct, change_24h_pct, change_7d_pct,
                refreshed_at
         FROM coins
         WHERE symbol = $1",
    )
    .bind(symbol)
    .fetch_optional(db)
    .await?;
    Ok(row)
}
