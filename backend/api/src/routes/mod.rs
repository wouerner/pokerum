use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::AppState;

pub mod engines;
pub mod jobs;
pub mod resumes;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(engines::router())
        .merge(jobs::router())
        .merge(resumes::router())
        .fallback(not_found)
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

async fn not_found() -> crate::error::AppError {
    crate::error::AppError::NotFound("rota não encontrada".into())
}
