use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};

use crate::error::AppResult;
use crate::models::ResumeView;
use crate::{service, AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/resumes/:codename", get(show).delete(remove))
}

async fn show(
    State(st): State<AppState>,
    Path(codename): Path<String>,
) -> AppResult<Json<ResumeView>> {
    Ok(Json(service::resume_view(&st.pool, &codename).await?))
}

async fn remove(
    State(st): State<AppState>,
    Path(codename): Path<String>,
    headers: HeaderMap,
) -> AppResult<StatusCode> {
    let token = headers.get("x-owner-token").and_then(|v| v.to_str().ok());
    service::delete_resume(&st.pool, &codename, token).await?;
    Ok(StatusCode::NO_CONTENT)
}
