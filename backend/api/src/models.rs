//! DTOs da API REST (ver CONTRACT.md).

use pokerum_core::scoring::{CriterionScore, ScoreReport};
use serde::{Deserialize, Serialize};

use crate::engines::{EngineKind, Execution};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct JobSummary {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub company: Option<String>,
    pub created_at: String,
    pub applicants: i64,
    pub ranking_engine: String,
    /// Os 3 melhores colocados da vaga (preenchido em `service::list_jobs`).
    #[sqlx(skip)]
    #[serde(default)]
    pub top: Vec<TopCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TopCandidate {
    pub position: i64,
    pub codename: String,
    pub alias: String,
    pub score: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateJob {
    pub title: String,
    #[serde(default)]
    pub company: Option<String>,
    pub description: String,
    /// Motor que ordena o ranking da vaga (padrão `heuristic-v1`).
    #[serde(default)]
    pub ranking_engine: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub company: Option<String>,
    pub description: String,
    pub created_at: String,
    pub keywords: Vec<String>,
    pub ranking_engine: String,
    /// Motores ativos (que avaliam candidaturas novas).
    pub engines: Vec<EngineView>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RankingEntry {
    pub position: i64,
    pub codename: String,
    pub alias: String,
    pub score: f64,
    pub breakdown: Vec<CriterionScore>,
    pub created_at: String,
    pub engine_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyResponse {
    pub codename: String,
    pub alias: String,
    pub owner_token: String,
    pub score: f64,
    pub position: i64,
    pub total: i64,
    pub report: ScoreReport,
    pub anonymized_text: String,
    pub primary_engine: String,
    pub evaluations: Vec<Evaluation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResumeApplication {
    pub job_slug: String,
    pub job_title: String,
    pub score: f64,
    pub position: i64,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResumeView {
    pub codename: String,
    pub alias: String,
    pub anonymized_text: String,
    pub applications: Vec<ResumeApplication>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationDetail {
    pub codename: String,
    pub alias: String,
    pub score: f64,
    pub position: i64,
    pub total: i64,
    pub report: ScoreReport,
    pub primary_engine: String,
    pub evaluations: Vec<Evaluation>,
}

/// Motor de avaliação como exposto pela API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineView {
    pub id: String,
    pub name: String,
    pub kind: EngineKind,
    pub description: String,
    pub execution: Execution,
    /// Ligado no banco (`scoring_engines.enabled`).
    pub enabled: bool,
    /// Configurado (ex.: API key presente).
    pub available: bool,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationStatus {
    Pending,
    Running,
    Done,
    Failed,
}

/// Resultado de um motor para uma candidatura.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub engine_id: String,
    pub engine_name: String,
    pub engine_kind: EngineKind,
    pub status: EvaluationStatus,
    pub score: Option<f64>,
    /// Posição no ranking deste motor (só quando `done`).
    pub position: Option<i64>,
    pub total: Option<i64>,
    pub report: Option<ScoreReport>,
    pub error: Option<String>,
    pub updated_at: String,
}

/// Linha da tabela `jobs`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JobRow {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub company: Option<String>,
    pub description: String,
    pub profile_json: String,
    pub created_at: String,
    pub ranking_engine: String,
}
