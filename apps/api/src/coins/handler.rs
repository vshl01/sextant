//! HTTP handlers for /coins. Pure DB reads — the fetcher does the work.

use axum::{
    Json,
    extract::{Path, State},
};

use crate::coins::service::{self, Coin};
use crate::error::AppError;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Coin>>, AppError> {
    let coins = service::list_all(&state.db).await?;
    Ok(Json(coins))
}

pub async fn get(
    State(state): State<AppState>,
    Path(symbol): Path<String>,
) -> Result<Json<Coin>, AppError> {
    let symbol = symbol.to_uppercase();
    match service::get(&state.db, &symbol).await? {
        Some(c) => Ok(Json(c)),
        None => Err(AppError::NotFound(format!("coin {symbol} not tracked"))),
    }
}
