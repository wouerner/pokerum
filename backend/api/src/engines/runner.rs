//! Executor de avaliações: cria as linhas em `evaluations`, roda cada motor
//! (inline na requisição ou em segundo plano), aplica timeout/retentativas e
//! retoma o que ficou pendente quando o servidor reinicia.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::{time::Duration, time::Instant};

use sqlx::SqlitePool;
use tokio::sync::Semaphore;

use super::{
    sanitize_report, EngineError, EngineRegistry, EvaluationInput, Execution, ScoringEngine,
};
use crate::error::{AppError, AppResult};
use crate::models::EngineView;

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";

#[derive(Clone)]
pub struct Runner {
    pool: SqlitePool,
    registry: Arc<EngineRegistry>,
    /// Limita quantas avaliações em segundo plano rodam ao mesmo tempo.
    permits: Arc<Semaphore>,
    /// Base do backoff entre tentativas (menor nos testes).
    backoff: Duration,
    /// Avaliações sendo processadas por este processo (evita rodar a mesma duas vezes
    /// quando a varredura da fila encontra uma que está em backoff).
    inflight: Arc<Mutex<HashSet<i64>>>,
}

/// Marca uma avaliação como "em andamento neste processo" enquanto existir.
struct InflightGuard {
    set: Arc<Mutex<HashSet<i64>>>,
    id: i64,
}

impl Drop for InflightGuard {
    fn drop(&mut self) {
        if let Ok(mut s) = self.set.lock() {
            s.remove(&self.id);
        }
    }
}

impl Runner {
    pub fn new(pool: SqlitePool, registry: EngineRegistry, concurrency: usize) -> Self {
        Self {
            pool,
            registry: Arc::new(registry),
            permits: Arc::new(Semaphore::new(concurrency.max(1))),
            backoff: Duration::from_secs(2),
            inflight: Arc::default(),
        }
    }

    pub fn with_backoff(mut self, backoff: Duration) -> Self {
        self.backoff = backoff;
        self
    }

    pub fn registry(&self) -> &EngineRegistry {
        &self.registry
    }

    /// Grava/atualiza no banco os metadados dos motores registrados em código.
    /// `enabled` e `config_json` não são tocados (são controlados no banco).
    pub async fn sync_catalog(&self) -> anyhow::Result<()> {
        for engine in self.registry.all() {
            let info = engine.info();
            sqlx::query(&format!(
                "INSERT INTO scoring_engines (id, name, kind, description, execution)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name, kind = excluded.kind,
                    description = excluded.description, execution = excluded.execution,
                    updated_at = {NOW}"
            ))
            .bind(info.id)
            .bind(info.name)
            .bind(info.kind.as_str())
            .bind(info.description)
            .bind(info.execution.as_str())
            .execute(&self.pool)
            .await?;
        }
        // Motor que saiu do código fica desligado: suas notas antigas somem da API.
        // Se ele voltar a ser registrado, continua desligado até `engines enable <id>`.
        let ids: Vec<&str> = self.registry.all().map(|e| e.info().id).collect();
        let placeholders = vec!["?"; ids.len()].join(", ");
        let sql = format!(
            "UPDATE scoring_engines SET enabled = 0, updated_at = {NOW}
             WHERE enabled = 1 AND id NOT IN ({placeholders})"
        );
        let mut q = sqlx::query(&sql);
        for id in &ids {
            q = q.bind(*id);
        }
        let retired = q.execute(&self.pool).await?.rows_affected();
        if retired > 0 {
            tracing::info!(retired, "motores fora do código desligados");
        }
        Ok(())
    }

    /// Todos os motores registrados, com estado (ligado no banco / configurado).
    pub async fn engines(&self) -> AppResult<Vec<EngineView>> {
        let enabled: std::collections::HashMap<String, bool> =
            sqlx::query_as::<_, (String, bool)>("SELECT id, enabled FROM scoring_engines")
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .collect();
        Ok(self
            .registry
            .all()
            .map(|e| {
                let info = e.info();
                let availability = e.availability();
                EngineView {
                    id: info.id.to_string(),
                    name: info.name.to_string(),
                    kind: info.kind,
                    description: info.description.to_string(),
                    execution: info.execution,
                    enabled: enabled.get(info.id).copied().unwrap_or(false),
                    available: availability.is_ok(),
                    unavailable_reason: availability.err(),
                }
            })
            .collect())
    }

    /// Motores que de fato avaliam: registrados, ligados e configurados.
    pub async fn active(&self) -> AppResult<Vec<Arc<dyn ScoringEngine>>> {
        Ok(self
            .engines()
            .await?
            .into_iter()
            .filter(|v| v.enabled && v.available)
            .filter_map(|v| self.registry.get(&v.id))
            .collect())
    }

    pub async fn is_active(&self, id: &str) -> AppResult<bool> {
        Ok(self.active().await?.iter().any(|e| e.info().id == id))
    }

    pub async fn set_enabled(&self, id: &str, enabled: bool) -> AppResult<()> {
        if !enabled && id == super::DEFAULT_ENGINE {
            return Err(AppError::BadRequest(format!(
                "{id} é o motor padrão (fallback de todas as vagas) e não pode ser desligado"
            )));
        }
        let r = sqlx::query(&format!(
            "UPDATE scoring_engines SET enabled = ?, updated_at = {NOW} WHERE id = ?"
        ))
        .bind(enabled)
        .bind(id)
        .execute(&self.pool)
        .await?;
        if r.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("motor '{id}' não existe")));
        }
        Ok(())
    }

    /// Cria as avaliações de uma candidatura nova para todos os motores ativos.
    /// Motores inline terminam antes de retornar; os de segundo plano ficam na fila.
    pub async fn evaluate_application(
        &self,
        application_id: i64,
        input: Arc<EvaluationInput>,
    ) -> AppResult<()> {
        for engine in self.active().await? {
            let id = engine.info().id;
            sqlx::query(
                "INSERT OR IGNORE INTO evaluations (application_id, engine_id) VALUES (?, ?)",
            )
            .bind(application_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
            let eval_id: i64 = sqlx::query_scalar(
                "SELECT id FROM evaluations WHERE application_id = ? AND engine_id = ?",
            )
            .bind(application_id)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
            match engine.info().execution {
                Execution::Inline => {
                    if let Some(_guard) = self.claim_local(eval_id) {
                        self.run(eval_id, engine, input.clone()).await;
                    }
                }
                Execution::Background => {
                    self.spawn(eval_id, engine, input.clone());
                }
            }
        }
        Ok(())
    }

    fn claim_local(&self, eval_id: i64) -> Option<InflightGuard> {
        let mut set = self.inflight.lock().ok()?;
        set.insert(eval_id).then(|| InflightGuard {
            set: self.inflight.clone(),
            id: eval_id,
        })
    }

    /// Roda em segundo plano (respeitando o limite de concorrência). Não faz nada se a
    /// avaliação já estiver em andamento neste processo. Retorna se agendou.
    fn spawn(
        &self,
        eval_id: i64,
        engine: Arc<dyn ScoringEngine>,
        input: Arc<EvaluationInput>,
    ) -> bool {
        let Some(guard) = self.claim_local(eval_id) else {
            return false;
        };
        let me = self.clone();
        tokio::spawn(async move {
            let _guard = guard;
            let Ok(_permit) = me.permits.clone().acquire_owned().await else {
                return;
            };
            me.run(eval_id, engine, input).await;
        });
        true
    }

    /// Roda um motor para uma avaliação, com timeout e retentativas. Nunca entra em pânico
    /// por erro do motor: o erro é gravado em `evaluations.error`.
    async fn run(&self, eval_id: i64, engine: Arc<dyn ScoringEngine>, input: Arc<EvaluationInput>) {
        let engine_id = engine.info().id;
        let max_attempts = engine.max_attempts().max(1);
        loop {
            // marca como "running" (se a candidatura foi apagada, não há o que fazer)
            let claimed = sqlx::query(&format!(
                "UPDATE evaluations SET status = 'running', attempts = attempts + 1, updated_at = {NOW}
                 WHERE id = ? AND status IN ('pending', 'running')
                 RETURNING attempts"
            ))
            .bind(eval_id)
            .fetch_optional(&self.pool)
            .await;
            let attempt: i64 = match claimed {
                Ok(Some(row)) => sqlx::Row::get(&row, 0),
                Ok(None) => return,
                Err(e) => {
                    tracing::error!(eval_id, engine_id, error = %e, "não foi possível iniciar avaliação");
                    return;
                }
            };

            let started = Instant::now();
            let result = match tokio::time::timeout(engine.timeout(), engine.evaluate(&input)).await
            {
                Ok(r) => r.and_then(sanitize_report),
                Err(_) => Err(EngineError::Transient(format!(
                    "tempo esgotado ({}s)",
                    engine.timeout().as_secs()
                ))),
            };
            let duration_ms = started.elapsed().as_millis() as i64;

            let outcome = match result {
                Ok(report) => {
                    let json = match serde_json::to_string(&report) {
                        Ok(j) => j,
                        Err(e) => {
                            self.fail(eval_id, &format!("relatório inválido: {e}"), duration_ms)
                                .await;
                            return;
                        }
                    };
                    sqlx::query(&format!(
                        "UPDATE evaluations SET status = 'done', score = ?, report_json = ?, error = NULL,
                            duration_ms = ?, updated_at = {NOW}, finished_at = {NOW}
                         WHERE id = ?"
                    ))
                    .bind(report.total)
                    .bind(json)
                    .bind(duration_ms)
                    .bind(eval_id)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                }
                Err(e) if e.is_retryable() && (attempt as u32) < max_attempts => {
                    tracing::warn!(eval_id, engine_id, attempt, error = %e, "avaliação falhou; tentando de novo");
                    let _ = sqlx::query(&format!(
                        "UPDATE evaluations SET status = 'pending', error = ?, updated_at = {NOW} WHERE id = ?"
                    ))
                    .bind(e.to_string())
                    .bind(eval_id)
                    .execute(&self.pool)
                    .await;
                    let wait = self.backoff * 2u32.saturating_pow(attempt as u32 - 1);
                    tokio::time::sleep(wait.min(Duration::from_secs(60))).await;
                    continue;
                }
                Err(e) => {
                    tracing::warn!(eval_id, engine_id, attempt, error = %e, "avaliação falhou");
                    self.fail(eval_id, &e.to_string(), duration_ms).await;
                    Ok(())
                }
            };
            if let Err(e) = outcome {
                tracing::error!(eval_id, engine_id, error = %e, "erro ao gravar avaliação");
            }
            return;
        }
    }

    async fn fail(&self, eval_id: i64, error: &str, duration_ms: i64) {
        let r = sqlx::query(&format!(
            "UPDATE evaluations SET status = 'failed', error = ?, duration_ms = ?,
                updated_at = {NOW}, finished_at = {NOW}
             WHERE id = ?"
        ))
        .bind(error)
        .bind(duration_ms)
        .bind(eval_id)
        .execute(&self.pool)
        .await;
        if let Err(e) = r {
            tracing::error!(eval_id, error = %e, "erro ao gravar falha da avaliação");
        }
    }

    /// No start do servidor: o que estava `running` foi interrompido; volta para a fila
    /// e processa tudo o que está pendente.
    pub async fn resume_pending(&self) -> anyhow::Result<usize> {
        sqlx::query("UPDATE evaluations SET status = 'pending' WHERE status = 'running'")
            .execute(&self.pool)
            .await?;
        self.pump().await
    }

    /// Agenda as avaliações `pending` de motores ativos que ainda não estão em andamento.
    /// Avaliações de motores desligados ficam pendentes até o motor voltar.
    pub async fn pump(&self) -> anyhow::Result<usize> {
        let active: Vec<Arc<dyn ScoringEngine>> = self.active().await?;
        let pending: Vec<(i64, i64, String)> = sqlx::query_as(
            "SELECT id, application_id, engine_id FROM evaluations
             WHERE status = 'pending' ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut n = 0;
        for (eval_id, application_id, engine_id) in pending {
            let Some(engine) = active.iter().find(|e| e.info().id == engine_id).cloned() else {
                continue;
            };
            if self
                .inflight
                .lock()
                .map(|s| s.contains(&eval_id))
                .unwrap_or(true)
            {
                continue;
            }
            let input = Arc::new(load_input(&self.pool, application_id).await?);
            if self.spawn(eval_id, engine, input) {
                n += 1;
            }
        }
        Ok(n)
    }

    /// Varre a fila periodicamente (pega o que foi enfileirado por `backfill`/`retry`
    /// pela CLI ou por outro processo).
    pub fn start_queue_worker(&self, every: Duration) -> tokio::task::JoinHandle<()> {
        let me = self.clone();
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(every);
            tick.tick().await;
            loop {
                tick.tick().await;
                match me.pump().await {
                    Ok(0) => {}
                    Ok(n) => tracing::info!(n, "fila: avaliações agendadas"),
                    Err(e) => tracing::error!(error = %e, "fila: erro na varredura"),
                }
            }
        })
    }

    /// Motores inline são grátis e instantâneos: avaliam automaticamente as candidaturas
    /// antigas que ainda não têm nota deles. Motores em segundo plano (LLM/API, com custo)
    /// só fazem isso via `backfill` explícito.
    pub async fn backfill_inline(&self) -> anyhow::Result<u64> {
        let mut total = 0;
        for engine in self.active().await? {
            let info = engine.info();
            if info.execution == Execution::Inline {
                let n = self.backfill(info.id).await?;
                if n > 0 {
                    tracing::info!(engine = info.id, n, "backfill automático de motor inline");
                }
                total += n;
            }
        }
        Ok(total)
    }

    /// Enfileira o motor para todas as candidaturas que ainda não têm avaliação dele
    /// (ex.: motor novo ligado depois). Retorna quantas foram enfileiradas.
    pub async fn backfill(&self, engine_id: &str) -> AppResult<u64> {
        if self.registry.get(engine_id).is_none() {
            return Err(AppError::NotFound(format!(
                "motor '{engine_id}' não existe"
            )));
        }
        let r = sqlx::query(
            "INSERT INTO evaluations (application_id, engine_id)
             SELECT a.id, ? FROM applications a
             WHERE NOT EXISTS (SELECT 1 FROM evaluations e
                               WHERE e.application_id = a.id AND e.engine_id = ?)",
        )
        .bind(engine_id)
        .bind(engine_id)
        .execute(&self.pool)
        .await?;
        Ok(r.rows_affected())
    }

    /// Volta avaliações `failed` de um motor para a fila.
    pub async fn retry_failed(&self, engine_id: &str) -> AppResult<u64> {
        let r = sqlx::query(&format!(
            "UPDATE evaluations SET status = 'pending', attempts = 0, updated_at = {NOW}
             WHERE engine_id = ? AND status = 'failed'"
        ))
        .bind(engine_id)
        .execute(&self.pool)
        .await?;
        Ok(r.rows_affected())
    }
}

/// Monta a entrada de um motor a partir do banco (vaga + currículo anonimizado).
pub async fn load_input(pool: &SqlitePool, application_id: i64) -> anyhow::Result<EvaluationInput> {
    let row: (String, String, String, String, String, String) = sqlx::query_as(
        "SELECT j.slug, j.title, j.description, j.profile_json, r.codename, r.anonymized_text
         FROM applications a
         JOIN jobs j ON j.id = a.job_id
         JOIN resumes r ON r.id = a.resume_id
         WHERE a.id = ?",
    )
    .bind(application_id)
    .fetch_one(pool)
    .await?;
    let (slug, title, description, profile_json, codename, text) = row;
    Ok(EvaluationInput {
        job_profile: crate::service::parse_profile(&slug, &title, &description, &profile_json),
        job_title: title,
        job_description: description,
        resume_codename: codename,
        resume_text: text,
    })
}
