use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    timestamp: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "crypto_backend",
        version: env!("CARGO_PKG_VERSION"),
        timestamp: Utc::now().to_rfc3339(),
    })
}

#[derive(Deserialize)]
struct CreateNote {
    title: String,
    body: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct Note {
    id: Uuid,
    title: String,
    body: String,
    created_at: DateTime<Utc>,
}

async fn create_note(State(state): State<AppState>, Json(payload): Json<CreateNote>) -> Json<Note> {
    let note = sqlx::query_as::<_, Note>(
        "INSERT INTO notes (id, title, body) VALUES ($1, $2, $3)
         RETURNING id, title, body, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(&payload.title)
    .bind(&payload.body)
    .fetch_one(&state.db)
    .await
    .unwrap();

    Json(note)
}

async fn list_notes(State(state): State<AppState>) -> Json<Vec<Note>> {
    let notes = sqlx::query_as::<_, Note>(
        "SELECT id, title, body, created_at FROM notes ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await
    .unwrap();

    Json(notes)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // db
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // db pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    println!("🐘 Connected to Docker PostgreSQL successfully");

    // Quick-and-dirty migration. Replace with sqlx-cli migrations later.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id UUID PRIMARY KEY,
            title TEXT NOT NULL,
            body TEXT NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create notes table");

    let state = AppState { db: pool };

    let app = Router::new()
        .route("/health", get(health))
        .route("/notes", post(create_note).get(list_notes))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Server running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
