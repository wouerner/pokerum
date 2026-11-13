//! Pokerum API: axum + sqlx (SQLite) sobre `pokerum_core`.

pub mod analytics;
pub mod db;
pub mod engines;
pub mod error;
pub mod extract;
pub mod models;
pub mod routes;
pub mod seed;
pub mod service;
pub mod slug;

use std::{path::Path, sync::Arc};

use axum::{
    extract::DefaultBodyLimit,
    handler::HandlerWithoutStateExt,
    http::header,
    routing::get,
    Router,
};
use sqlx::SqlitePool;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

/// Limite de upload do currículo (bytes).
pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub runner: engines::Runner,
}

impl AppState {
    /// Estado com os motores embutidos ([`engines::EngineRegistry::builtin`]).
    pub async fn new(pool: SqlitePool) -> anyhow::Result<Self> {
        Self::with_registry(pool, engines::EngineRegistry::builtin(), 4).await
    }

    /// Sincroniza o catálogo de motores no banco e retoma avaliações pendentes.
    pub async fn with_registry(
        pool: SqlitePool,
        registry: engines::EngineRegistry,
        concurrency: usize,
    ) -> anyhow::Result<Self> {
        let runner = engines::Runner::new(pool.clone(), registry, concurrency);
        Self::with_runner(pool, runner).await
    }

    pub async fn with_runner(pool: SqlitePool, runner: engines::Runner) -> anyhow::Result<Self> {
        runner.sync_catalog().await?;
        runner.backfill_inline().await?;
        let resumed = runner.resume_pending().await?;
        if resumed > 0 {
            tracing::info!(resumed, "avaliações pendentes retomadas");
        }
        Ok(Self { pool, runner })
    }
}

/// Router da API (rotas sob `/api`), com tracing. CORS só se `POKERUM_CORS_ORIGINS` pedir:
/// em produção site e API têm a mesma origem, e no dev o Vite faz proxy de `/api`.
pub fn build_app(state: AppState) -> Router {
    let app = Router::new()
        .nest("/api", routes::router())
        .with_state(state)
        // margem para o overhead do multipart; o arquivo em si é checado contra MAX_UPLOAD_BYTES
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES + 256 * 1024));
    let app = match cors_layer(std::env::var("POKERUM_CORS_ORIGINS").ok().as_deref()) {
        Some(cors) => app.layer(cors),
        None => app,
    };
    app.layer(TraceLayer::new_for_http())
}

/// `*` libera qualquer origem; `https://a.com,https://b.com` libera só essas; vazio: sem CORS.
fn cors_layer(spec: Option<&str>) -> Option<CorsLayer> {
    let spec = spec.map(str::trim).filter(|s| !s.is_empty())?;
    if spec == "*" {
        return Some(CorsLayer::permissive());
    }
    let origins: Vec<axum::http::HeaderValue> = spec
        .split(',')
        .filter_map(|o| o.trim().parse().ok())
        .collect();
    Some(
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any),
    )
}

/// Adiciona o serviço de arquivos estáticos do SPA (com fallback para index.html).
/// Com `ga_id`, o index.html é servido da memória com a tag do Google Analytics no `<head>`.
pub fn with_static(app: Router, dist: &Path, ga_id: Option<&str>) -> Router {
    let index = dist.join("index.html");
    let html = ga_id.and_then(|id| match std::fs::read_to_string(&index) {
        Ok(html) => Some(analytics::inject(&html, id)),
        Err(e) => {
            tracing::warn!(error = %e, "não li o index.html; analytics desligado");
            None
        }
    });
    let Some(html) = html else {
        let serve = ServeDir::new(dist).fallback(ServeFile::new(index));
        return app.fallback_service(serve);
    };

    let html: Arc<str> = html.into();
    let page = move || {
        let html = html.clone();
        async move {
            (
                [
                    (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                    (header::CACHE_CONTROL, "no-cache"),
                ],
                html.to_string(),
            )
        }
    };
    let serve = ServeDir::new(dist)
        .append_index_html_on_directories(false)
        .fallback(page.clone().into_service());
    app.route("/", get(page.clone()))
        .route("/index.html", get(page))
        .fallback_service(serve)
}
