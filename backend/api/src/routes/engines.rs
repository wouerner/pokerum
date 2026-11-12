use axum::{extract::State, routing::get, Json, Router};

use crate::error::AppResult;
use crate::models::EngineView;
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/engines", get(list))
}

async fn list(State(st): State<AppState>) -> AppResult<Json<Vec<EngineView>>> {
    Ok(Json(st.runner.engines().await?))
}
