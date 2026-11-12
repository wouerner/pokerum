use axum::{
    extract::{FromRequest, Multipart, Path, Query, Request, State},
    http::{header, StatusCode},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::extract::extract_text;
use crate::models::*;
use crate::{service, AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs", get(list).post(create))
        .route("/jobs/:slug", get(show))
        .route("/jobs/:slug/ranking", get(ranking))
        .route("/jobs/:slug/apply", axum::routing::post(apply))
        .route("/jobs/:slug/applications/:codename", get(application))
}

async fn list(State(st): State<AppState>) -> AppResult<Json<Vec<JobSummary>>> {
    Ok(Json(service::list_jobs(&st.pool).await?))
}

async fn create(
    State(st): State<AppState>,
    body: Result<Json<CreateJob>, axum::extract::rejection::JsonRejection>,
) -> AppResult<(StatusCode, Json<Job>)> {
    let Json(input) =
        body.map_err(|e| AppError::BadRequest(format!("JSON inválido: {}", e.body_text())))?;
    let job = service::create_job(&st.pool, &st.runner, input).await?;
    Ok((StatusCode::CREATED, Json(job)))
}

async fn show(State(st): State<AppState>, Path(slug): Path<String>) -> AppResult<Json<Job>> {
    let row = service::job_by_slug(&st.pool, &slug).await?;
    Ok(Json(service::job_view(&st.runner, &row).await?))
}

#[derive(Deserialize)]
struct RankingQuery {
    engine: Option<String>,
}

async fn ranking(
    State(st): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<RankingQuery>,
) -> AppResult<Json<Vec<RankingEntry>>> {
    let row = service::job_by_slug(&st.pool, &slug).await?;
    Ok(Json(
        service::ranking(&st.pool, &row, q.engine.as_deref()).await?,
    ))
}

async fn application(
    State(st): State<AppState>,
    Path((slug, codename)): Path<(String, String)>,
) -> AppResult<Json<ApplicationDetail>> {
    let row = service::job_by_slug(&st.pool, &slug).await?;
    Ok(Json(
        service::application_detail(&st.pool, &row, &codename).await?,
    ))
}

#[derive(Deserialize)]
struct JsonApply {
    text: String,
    #[serde(default)]
    name: Option<String>,
}

fn multipart_err(e: axum::extract::multipart::MultipartError) -> AppError {
    if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
        AppError::PayloadTooLarge("arquivo excede o limite de 5 MB".into())
    } else {
        AppError::BadRequest(format!("multipart inválido: {}", e.body_text()))
    }
}

/// Lê (texto, nome) de um multipart (`file` | `text`, `name?`) ou de JSON `{text, name?}`.
async fn read_apply_input(req: Request) -> AppResult<(String, Option<String>)> {
    let ct = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ct.starts_with("application/json") {
        let Json(body) = Json::<JsonApply>::from_request(req, &())
            .await
            .map_err(|e| AppError::BadRequest(format!("JSON inválido: {}", e.body_text())))?;
        return Ok((body.text, body.name));
    }
    if !ct.starts_with("multipart/form-data") {
        return Err(AppError::UnsupportedMedia(
            "envie multipart/form-data com `file` ou `text`".into(),
        ));
    }

    let mut mp = Multipart::from_request(req, &())
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart inválido: {}", e.body_text())))?;
    let mut file_text: Option<String> = None;
    let mut text: Option<String> = None;
    let mut name: Option<String> = None;
    while let Some(field) = mp.next_field().await.map_err(multipart_err)? {
        match field.name().unwrap_or("") {
            "file" => {
                let filename = field.file_name().map(str::to_string);
                let ctype = field.content_type().map(str::to_string);
                let bytes = field.bytes().await.map_err(multipart_err)?;
                if bytes.is_empty() && filename.as_deref().unwrap_or("").is_empty() {
                    continue; // input de arquivo vazio no formulário
                }
                file_text = Some(
                    extract_text(filename.as_deref(), ctype.as_deref(), bytes.to_vec()).await?,
                );
            }
            "text" => text = Some(field.text().await.map_err(multipart_err)?),
            "name" => name = Some(field.text().await.map_err(multipart_err)?),
            _ => {}
        }
    }
    let body = file_text
        .filter(|t| !t.trim().is_empty())
        .or(text.filter(|t| !t.trim().is_empty()))
        .ok_or_else(|| {
            AppError::BadRequest(
                "envie um arquivo (`file`) ou o texto (`text`) do currículo".into(),
            )
        })?;
    Ok((body, name))
}

async fn apply(
    State(st): State<AppState>,
    Path(slug): Path<String>,
    req: Request,
) -> AppResult<(StatusCode, Json<ApplyResponse>)> {
    let job = service::job_by_slug(&st.pool, &slug).await?;
    let (text, name) = read_apply_input(req).await?;
    let resp = service::apply(&st.pool, &st.runner, &job, &text, name.as_deref()).await?;
    Ok((StatusCode::CREATED, Json(resp)))
}
