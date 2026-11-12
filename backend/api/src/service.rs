//! Regras de negócio que dependem de `pokerum_core` + acesso ao banco.
//! As rotas HTTP e o seed chamam estas funções.

use std::collections::HashSet;

use pokerum_core::anonymize::anonymize;
use pokerum_core::scoring::{analyze_job, score, JobProfile, ScoreReport};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use std::sync::Arc;

use crate::engines::{EvaluationInput, Runner, DEFAULT_ENGINE};
use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::slug::job_slug;

/// Mínimo de caracteres de texto útil no currículo.
pub const MIN_RESUME_CHARS: usize = 200;

// ---------------------------------------------------------------- jobs

pub async fn create_job(pool: &SqlitePool, runner: &Runner, input: CreateJob) -> AppResult<Job> {
    let title = input.title.trim().to_string();
    let description = input.description.trim().to_string();
    let company = input
        .company
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty());

    if title.is_empty() || title.chars().count() > 200 {
        return Err(AppError::BadRequest(
            "título é obrigatório (até 200 caracteres)".into(),
        ));
    }
    if description.chars().count() < 20 {
        return Err(AppError::BadRequest(
            "descrição da vaga muito curta (mínimo 20 caracteres)".into(),
        ));
    }
    if description.len() > 100_000 {
        return Err(AppError::BadRequest("descrição da vaga muito longa".into()));
    }

    let ranking_engine = input
        .ranking_engine
        .map(|e| e.trim().to_string())
        .filter(|e| !e.is_empty())
        .unwrap_or_else(|| DEFAULT_ENGINE.to_string());
    if !runner.is_active(&ranking_engine).await? {
        return Err(AppError::BadRequest(format!(
            "algoritmo de ranking '{ranking_engine}' não existe ou não está ativo"
        )));
    }

    let profile = analyze_job(&title, &description);
    let profile_json = serde_json::to_string(&profile)?;

    for _ in 0..5 {
        let slug = {
            let mut rng = StdRng::from_entropy();
            job_slug(&title, &mut rng)
        };
        let res = sqlx::query(
            "INSERT INTO jobs (slug, title, company, description, profile_json, ranking_engine)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&slug)
        .bind(&title)
        .bind(&company)
        .bind(&description)
        .bind(&profile_json)
        .bind(&ranking_engine)
        .execute(pool)
        .await;
        match res {
            Ok(_) => {
                let row = job_by_slug(pool, &slug).await?;
                return job_view(runner, &row).await;
            }
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err(AppError::Internal(anyhow::anyhow!(
        "não foi possível gerar slug único"
    )))
}

pub async fn list_jobs(pool: &SqlitePool) -> AppResult<Vec<JobSummary>> {
    let rows = sqlx::query_as::<_, JobSummary>(
        "SELECT j.id, j.slug, j.title, j.company, j.created_at,
                COALESCE((SELECT s.id FROM scoring_engines s WHERE s.id = j.ranking_engine AND s.enabled = 1), 'heuristic-v1') AS ranking_engine,
                (SELECT COUNT(*) FROM applications a WHERE a.job_id = j.id) AS applicants
         FROM jobs j
         ORDER BY j.created_at DESC, j.id DESC",
    )
    .fetch_all(pool)
    .await?;
    let mut rows = rows;

    // Top 3 de cada vaga, pelo motor de ranking da vaga, numa única query.
    let top: Vec<(i64, TopCandidate)> = sqlx::query_as::<_, (i64, i64, String, String, f64)>(
        "SELECT job_id, position, codename, alias, score FROM (
            SELECT a.job_id, r.codename, r.alias, e.score,
                   ROW_NUMBER() OVER (PARTITION BY a.job_id
                                      ORDER BY e.score DESC, a.created_at ASC, a.id ASC) AS position
            FROM evaluations e
            JOIN applications a ON a.id = e.application_id
            JOIN jobs j ON j.id = a.job_id
            JOIN resumes r ON r.id = a.resume_id
            WHERE e.engine_id = COALESCE((SELECT s.id FROM scoring_engines s WHERE s.id = j.ranking_engine AND s.enabled = 1), 'heuristic-v1') AND e.status = 'done'
         ) WHERE position <= 3
         ORDER BY job_id, position",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(job_id, position, codename, alias, score)| {
        (
            job_id,
            TopCandidate {
                position,
                codename,
                alias,
                score,
            },
        )
    })
    .collect();

    let mut by_job: std::collections::HashMap<i64, Vec<TopCandidate>> = Default::default();
    for (job_id, c) in top {
        by_job.entry(job_id).or_default().push(c);
    }
    for job in &mut rows {
        job.top = by_job.remove(&job.id).unwrap_or_default();
    }
    Ok(rows)
}

pub async fn job_by_slug(pool: &SqlitePool, slug: &str) -> AppResult<JobRow> {
    sqlx::query_as::<_, JobRow>(
        // motor de ranking efetivo: o da vaga se ainda estiver ligado, senão o padrão
        "SELECT id, slug, title, company, description, profile_json, created_at,
                COALESCE((SELECT s.id FROM scoring_engines s WHERE s.id = jobs.ranking_engine AND s.enabled = 1), 'heuristic-v1') AS ranking_engine
         FROM jobs WHERE slug = ?",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("vaga não encontrada".into()))
}

pub async fn job_view(runner: &Runner, row: &JobRow) -> AppResult<Job> {
    let engines = runner
        .engines()
        .await?
        .into_iter()
        .filter(|e| e.enabled && e.available)
        .collect();
    Ok(Job {
        id: row.id,
        slug: row.slug.clone(),
        title: row.title.clone(),
        company: row.company.clone(),
        description: row.description.clone(),
        created_at: row.created_at.clone(),
        keywords: load_profile(row).keywords,
        ranking_engine: row.ranking_engine.clone(),
        engines,
    })
}

fn load_profile(row: &JobRow) -> JobProfile {
    parse_profile(&row.slug, &row.title, &row.description, &row.profile_json)
}

/// Lê o `profile_json` da vaga; se for de versão antiga do core, reanalisa.
pub fn parse_profile(slug: &str, title: &str, description: &str, profile_json: &str) -> JobProfile {
    match serde_json::from_str::<JobProfile>(profile_json) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(%slug, error = %e, "profile_json inválido; reanalisando");
            analyze_job(title, description)
        }
    }
}

// ---------------------------------------------------------------- apply

pub fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub fn verify_token(token: &str, stored_hash: &str) -> bool {
    constant_time_eq(hash_token(token).as_bytes(), stored_hash.as_bytes())
}

/// Conta caracteres "úteis" (não-espaço) do texto.
fn meaningful_chars(s: &str) -> usize {
    s.chars().filter(|c| !c.is_whitespace()).count()
}

/// Anonimiza, pontua e grava a candidatura. O texto original e o nome real
/// NUNCA são persistidos.
pub async fn apply(
    pool: &SqlitePool,
    runner: &Runner,
    job: &JobRow,
    raw_text: &str,
    declared_name: Option<&str>,
) -> AppResult<ApplyResponse> {
    let raw_text = raw_text.trim();
    if meaningful_chars(raw_text) < MIN_RESUME_CHARS {
        return Err(AppError::BadRequest(format!(
            "currículo muito curto: são necessários pelo menos {MIN_RESUME_CHARS} caracteres de texto"
        )));
    }
    let declared_name = declared_name.map(str::trim).filter(|n| !n.is_empty());
    let profile = load_profile(job);

    for attempt in 0..5 {
        let taken: HashSet<String> =
            sqlx::query_scalar::<_, String>("SELECT codename FROM resumes")
                .fetch_all(pool)
                .await?
                .into_iter()
                .collect();

        let (anon, owner_token) = {
            let mut rng = StdRng::from_entropy();
            let anon = anonymize(raw_text, declared_name, &taken, &mut rng);
            let mut tok = [0u8; 32];
            rng.fill_bytes(&mut tok);
            (anon, hex::encode(tok))
        };
        // legado: applications.score guarda o heuristic-v1 (ver migration 0002)
        let report: ScoreReport = score(&profile, &anon.text);
        let total_score = report.total.clamp(0.0, 100.0);
        let redactions_json = serde_json::to_string(
            &anon
                .redactions
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "kind": format!("{:?}", r.kind),
                        "count": r.count,
                    })
                })
                .collect::<Vec<_>>(),
        )?;
        let report_json = serde_json::to_string(&report)?;

        let mut tx = pool.begin().await?;
        let ins = sqlx::query(
            "INSERT INTO resumes (codename, alias, anonymized_text, redactions_json, owner_token_hash)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&anon.codename)
        .bind(&anon.alias)
        .bind(&anon.text)
        .bind(&redactions_json)
        .bind(hash_token(&owner_token))
        .execute(&mut *tx)
        .await;
        let resume_id = match ins {
            Ok(r) => r.last_insert_rowid(),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                tracing::warn!(attempt, codename = %anon.codename, "codinome em uso; tentando de novo");
                tx.rollback().await?;
                continue;
            }
            Err(e) => return Err(e.into()),
        };
        let application_id = sqlx::query(
            "INSERT INTO applications (job_id, resume_id, score, report_json, engine_version)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(job.id)
        .bind(resume_id)
        .bind(total_score)
        .bind(&report_json)
        .bind(&report.engine_version)
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        tx.commit().await?;

        // Avalia com todos os motores ativos (inline agora; background na fila).
        let input = Arc::new(EvaluationInput {
            job_title: job.title.clone(),
            job_description: job.description.clone(),
            job_profile: profile.clone(),
            resume_codename: anon.codename.clone(),
            resume_text: anon.text.clone(),
        });
        runner.evaluate_application(application_id, input).await?;

        let d = application_detail(pool, job, &anon.codename).await?;
        return Ok(ApplyResponse {
            codename: d.codename,
            alias: d.alias,
            owner_token,
            score: d.score,
            position: d.position,
            total: d.total,
            report: d.report,
            anonymized_text: anon.text,
            primary_engine: d.primary_engine,
            evaluations: d.evaluations,
        });
    }
    Err(AppError::Conflict(
        "não foi possível gerar um codinome único; tente novamente".into(),
    ))
}

// ---------------------------------------------------------------- ranking

/// Ranking de uma vaga por um motor: só avaliações `done`.
const RANKED: &str =
    "SELECT a.id AS application_id, a.resume_id, e.score, e.report_json, a.created_at,
            ROW_NUMBER() OVER (ORDER BY e.score DESC, a.created_at ASC, a.id ASC) AS position,
            COUNT(*) OVER () AS total
     FROM evaluations e JOIN applications a ON a.id = e.application_id
     WHERE a.job_id = ? AND e.engine_id = ? AND e.status = 'done'";

/// Posição/nota de uma candidatura no ranking de um motor (se já avaliada por ele).
async fn ranked_row(
    pool: &SqlitePool,
    job_id: i64,
    engine_id: &str,
    application_id: i64,
) -> AppResult<Option<(f64, String, i64, i64)>> {
    let sql = format!(
        "SELECT score, report_json, position, total FROM ({RANKED}) WHERE application_id = ?"
    );
    Ok(sqlx::query_as(&sql)
        .bind(job_id)
        .bind(engine_id)
        .bind(application_id)
        .fetch_optional(pool)
        .await?)
}

/// Resultado "principal" de uma candidatura: motor de ranking da vaga; se ele ainda
/// não terminou, o heuristic-v1 (inline, sempre disponível).
async fn primary_result(
    pool: &SqlitePool,
    job_id: i64,
    ranking_engine: &str,
    application_id: i64,
) -> AppResult<(String, f64, ScoreReport, i64, i64)> {
    for engine in [ranking_engine, DEFAULT_ENGINE] {
        if let Some((score, report_json, position, total)) =
            ranked_row(pool, job_id, engine, application_id).await?
        {
            return Ok((
                engine.to_string(),
                score,
                serde_json::from_str(&report_json)?,
                position,
                total,
            ));
        }
    }
    Err(AppError::Conflict(
        "candidatura ainda não foi avaliada".into(),
    ))
}

pub async fn ranking(
    pool: &SqlitePool,
    job: &JobRow,
    engine: Option<&str>,
) -> AppResult<Vec<RankingEntry>> {
    let engine_id = engine
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .unwrap_or(&job.ranking_engine);
    let exists: bool =
        sqlx::query_scalar("SELECT COUNT(*) > 0 FROM scoring_engines WHERE id = ? AND enabled = 1")
            .bind(engine_id)
            .fetch_one(pool)
            .await?;
    if !exists {
        return Err(AppError::NotFound(format!(
            "algoritmo '{engine_id}' não existe ou está desligado"
        )));
    }
    let sql = format!(
        "SELECT r.position, res.codename, res.alias, r.score, r.report_json, r.created_at
         FROM ({RANKED}) r JOIN resumes res ON res.id = r.resume_id
         ORDER BY r.position ASC"
    );
    let rows: Vec<(i64, String, String, f64, String, String)> = sqlx::query_as(&sql)
        .bind(job.id)
        .bind(engine_id)
        .fetch_all(pool)
        .await?;
    rows.into_iter()
        .map(
            |(position, codename, alias, score, report_json, created_at)| {
                let report: ScoreReport = serde_json::from_str(&report_json)?;
                Ok(RankingEntry {
                    position,
                    codename,
                    alias,
                    score,
                    breakdown: report.breakdown,
                    created_at,
                    engine_id: engine_id.to_string(),
                })
            },
        )
        .collect()
}

/// Todas as avaliações de uma candidatura (motor de ranking da vaga primeiro).
async fn evaluations_of(
    pool: &SqlitePool,
    job: &JobRow,
    application_id: i64,
) -> AppResult<Vec<Evaluation>> {
    type Row = (
        String,
        String,
        String,
        String,
        Option<f64>,
        Option<String>,
        Option<String>,
        String,
        Option<i64>,
        Option<i64>,
    );
    let rows: Vec<Row> = sqlx::query_as(
        "WITH ranked AS (
            SELECT e.id,
                   ROW_NUMBER() OVER (PARTITION BY e.engine_id
                                      ORDER BY e.score DESC, a.created_at ASC, a.id ASC) AS position,
                   COUNT(*) OVER (PARTITION BY e.engine_id) AS total
            FROM evaluations e JOIN applications a ON a.id = e.application_id
            WHERE a.job_id = ? AND e.status = 'done'
         )
         SELECT e.engine_id, s.name, s.kind, e.status, e.score, e.report_json, e.error,
                e.updated_at, r.position, r.total
         FROM evaluations e
         JOIN scoring_engines s ON s.id = e.engine_id AND s.enabled = 1
         LEFT JOIN ranked r ON r.id = e.id
         WHERE e.application_id = ?
         ORDER BY (e.engine_id = ?) DESC, (e.engine_id = ?) DESC, s.name",
    )
    .bind(job.id)
    .bind(application_id)
    .bind(&job.ranking_engine)
    .bind(DEFAULT_ENGINE)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(
            |(
                engine_id,
                engine_name,
                kind,
                status,
                score,
                report_json,
                error,
                updated_at,
                position,
                total,
            )| {
                let status: EvaluationStatus =
                    serde_json::from_value(serde_json::Value::String(status))?;
                let engine_kind = serde_json::from_value(serde_json::Value::String(kind))?;
                let report = match (status, report_json) {
                    (EvaluationStatus::Done, Some(j)) => Some(serde_json::from_str(&j)?),
                    _ => None,
                };
                Ok(Evaluation {
                    engine_id,
                    engine_name,
                    engine_kind,
                    status,
                    score,
                    position,
                    total,
                    report,
                    // `error` de uma tentativa anterior não interessa se terminou bem
                    error: if status == EvaluationStatus::Done {
                        None
                    } else {
                        error
                    },
                    updated_at,
                })
            },
        )
        .collect()
}

pub async fn application_detail(
    pool: &SqlitePool,
    job: &JobRow,
    codename: &str,
) -> AppResult<ApplicationDetail> {
    let row: Option<(i64, String, String)> = sqlx::query_as(
        "SELECT a.id, r.codename, r.alias FROM applications a
         JOIN resumes r ON r.id = a.resume_id
         WHERE a.job_id = ? AND r.codename = ?",
    )
    .bind(job.id)
    .bind(codename)
    .fetch_optional(pool)
    .await?;
    let (application_id, codename, alias) =
        row.ok_or_else(|| AppError::NotFound("candidatura não encontrada".into()))?;

    let (primary_engine, score, report, position, total) =
        primary_result(pool, job.id, &job.ranking_engine, application_id).await?;
    Ok(ApplicationDetail {
        codename,
        alias,
        score,
        position,
        total,
        report,
        primary_engine,
        evaluations: evaluations_of(pool, job, application_id).await?,
    })
}

// ---------------------------------------------------------------- resumes

pub async fn resume_view(pool: &SqlitePool, codename: &str) -> AppResult<ResumeView> {
    let row: Option<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, codename, alias, anonymized_text FROM resumes WHERE codename = ?",
    )
    .bind(codename)
    .fetch_optional(pool)
    .await?;
    let (resume_id, codename, alias, anonymized_text) =
        row.ok_or_else(|| AppError::NotFound("currículo não encontrado".into()))?;

    let apps: Vec<(i64, i64, String, String, String)> = sqlx::query_as(
        "SELECT a.id, j.id, j.slug, j.title,
                COALESCE((SELECT s.id FROM scoring_engines s WHERE s.id = j.ranking_engine AND s.enabled = 1), 'heuristic-v1')
         FROM applications a
         JOIN jobs j ON j.id = a.job_id
         WHERE a.resume_id = ? ORDER BY a.created_at ASC",
    )
    .bind(resume_id)
    .fetch_all(pool)
    .await?;

    let mut applications = Vec::with_capacity(apps.len());
    for (application_id, job_id, job_slug, job_title, ranking_engine) in apps {
        let Ok((_, score, _, position, total)) =
            primary_result(pool, job_id, &ranking_engine, application_id).await
        else {
            continue; // ainda sem avaliação concluída
        };
        applications.push(ResumeApplication {
            job_slug,
            job_title,
            score,
            position,
            total,
        });
    }
    Ok(ResumeView {
        codename,
        alias,
        anonymized_text,
        applications,
    })
}

pub async fn delete_resume(
    pool: &SqlitePool,
    codename: &str,
    token: Option<&str>,
) -> AppResult<()> {
    let stored: Option<(i64, String)> =
        sqlx::query_as("SELECT id, owner_token_hash FROM resumes WHERE codename = ?")
            .bind(codename)
            .fetch_optional(pool)
            .await?;
    let Some((id, hash)) = stored else {
        return Err(AppError::NotFound("currículo não encontrado".into()));
    };
    let token = token.map(str::trim).unwrap_or("");
    if token.is_empty() || !verify_token(token, &hash) {
        return Err(AppError::Forbidden("código de exclusão inválido".into()));
    }
    sqlx::query("DELETE FROM resumes WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_roundtrip() {
        let h = hash_token("abc");
        assert!(verify_token("abc", &h));
        assert!(!verify_token("abd", &h));
    }
}
