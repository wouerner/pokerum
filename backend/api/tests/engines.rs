//! Testes da arquitetura de motores plugáveis, com motores falsos que simulam
//! LLM lenta, API instável, motor quebrado e motor sem configuração.

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::time::Duration;

use async_trait::async_trait;
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use pokerum_api::engines::{
    EngineError, EngineInfo, EngineKind, EngineRegistry, EvaluationInput, Execution, Runner,
    ScoringEngine,
};
use pokerum_core::scoring::{CriterionScore, ScoreReport};
use serde_json::{json, Value};
use tower::ServiceExt;

// ------------------------------------------------------------------ motores falsos

fn report(id: &str, total: f64) -> ScoreReport {
    ScoreReport {
        total,
        breakdown: vec![CriterionScore {
            key: "fit".into(),
            label: "Aderência".into(),
            weight: 1.0,
            score: total,
            detail: "teste".into(),
        }],
        matched_keywords: vec![],
        missing_keywords: vec![],
        suggestions: vec!["sugestão".into()],
        engine_version: id.into(),
    }
}

/// "LLM" lenta: roda em segundo plano e dá nota pelo tamanho do texto (determinística).
struct FakeLlm;
#[async_trait]
impl ScoringEngine for FakeLlm {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "fake-llm",
            name: "LLM de teste",
            kind: EngineKind::Llm,
            description: "simula uma LLM",
            execution: Execution::Background,
        }
    }
    async fn evaluate(&self, input: &EvaluationInput) -> Result<ScoreReport, EngineError> {
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(
            !input.resume_text.contains("joana@example.com"),
            "PII chegou ao motor"
        );
        let total = (input.resume_text.len() % 100) as f64;
        Ok(report("fake-llm", total))
    }
}

/// API instável: falha (transitório) na 1ª chamada de cada currículo e acerta depois.
struct FlakyApi(Arc<AtomicU32>);
#[async_trait]
impl ScoringEngine for FlakyApi {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "flaky-api",
            name: "API instável",
            kind: EngineKind::ExternalApi,
            description: "falha uma vez",
            execution: Execution::Background,
        }
    }
    async fn evaluate(&self, _: &EvaluationInput) -> Result<ScoreReport, EngineError> {
        if self.0.fetch_add(1, Ordering::SeqCst).is_multiple_of(2) {
            Err(EngineError::Transient("503 do provedor".into()))
        } else {
            Ok(report("flaky-api", 61.0))
        }
    }
}

/// Motor quebrado: erro permanente.
struct Broken;
#[async_trait]
impl ScoringEngine for Broken {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "broken-db",
            name: "Banco quebrado",
            kind: EngineKind::Database,
            description: "sempre falha",
            execution: Execution::Background,
        }
    }
    async fn evaluate(&self, _: &EvaluationInput) -> Result<ScoreReport, EngineError> {
        Err(EngineError::Permanent("resposta inválida".into()))
    }
}

/// Motor sem configuração (ex.: API key ausente): registrado, mas não avalia.
struct Unconfigured;
#[async_trait]
impl ScoringEngine for Unconfigured {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "needs-key",
            name: "Precisa de chave",
            kind: EngineKind::Llm,
            description: "sem API key",
            execution: Execution::Background,
        }
    }
    fn availability(&self) -> Result<(), String> {
        Err("defina FAKE_API_KEY".into())
    }
    async fn evaluate(&self, _: &EvaluationInput) -> Result<ScoreReport, EngineError> {
        unreachable!("motor indisponível não deve ser chamado")
    }
}

/// Motor que devolve nota fora da escala: deve ser corrigida para 0..=100.
struct OutOfRange;
#[async_trait]
impl ScoringEngine for OutOfRange {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: "out-of-range",
            name: "Fora da escala",
            kind: EngineKind::Heuristic,
            description: "nota 250",
            execution: Execution::Inline,
        }
    }
    async fn evaluate(&self, _: &EvaluationInput) -> Result<ScoreReport, EngineError> {
        Ok(report("out-of-range", 250.0))
    }
}

fn registry() -> EngineRegistry {
    let mut r = EngineRegistry::builtin();
    r.register(pokerum_api::engines::keywords::KeywordsEngine);
    r.register(FakeLlm)
        .register(FlakyApi(Arc::new(AtomicU32::new(0))))
        .register(Broken)
        .register(Unconfigured)
        .register(OutOfRange);
    r
}

async fn app_with(pool: sqlx::SqlitePool) -> Router {
    let runner = Runner::new(pool.clone(), registry(), 4).with_backoff(Duration::from_millis(10));
    let state = pokerum_api::AppState::with_runner(pool, runner)
        .await
        .unwrap();
    pokerum_api::build_app(state)
}

async fn app() -> Router {
    app_with(pokerum_api::db::connect("sqlite::memory:").await.unwrap()).await
}

// ------------------------------------------------------------------ helpers HTTP

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, v)
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).body(Body::empty()).unwrap()
}

fn post_json(uri: &str, body: Value) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

const JOB_DESC: &str =
    "Vaga para desenvolvedor backend Rust sênior. Requisitos: Rust, tokio, axum, \
PostgreSQL, Docker, Kubernetes, AWS. 5 anos de experiência. Inglês avançado. Desejável: Kafka.";

const CV: &str = "Joana Martins\njoana@example.com\n\nResumo\nEngenheira de software sênior com 8 anos \
de experiência em Rust, tokio, axum e PostgreSQL. Docker, Kubernetes e AWS no dia a dia.\n\nExperiência\n\
Empresa X — Backend Sênior (2018 - atual)\n- APIs em Rust com axum, redução de 40% na latência\n\
- Clusters Kubernetes na AWS\n\nFormação\nCiência da Computação\n\nIdiomas\nInglês fluente";

async fn create_job(app: &Router, ranking_engine: Option<&str>) -> String {
    let mut body = json!({"title": "Backend Rust Sênior", "description": JOB_DESC});
    if let Some(e) = ranking_engine {
        body["ranking_engine"] = json!(e);
    }
    let (st, v) = send(app, post_json("/api/jobs", body)).await;
    assert_eq!(st, StatusCode::CREATED, "{v}");
    v["slug"].as_str().unwrap().to_string()
}

async fn apply(app: &Router, slug: &str, text: &str) -> Value {
    let (st, v) = send(
        app,
        post_json(&format!("/api/jobs/{slug}/apply"), json!({"text": text})),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "{v}");
    v
}

fn eval<'a>(v: &'a Value, engine: &str) -> &'a Value {
    v["evaluations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["engine_id"] == engine)
        .unwrap_or_else(|| panic!("sem avaliação de {engine}: {v}"))
}

/// Faz polling da candidatura até não haver avaliação pendente.
async fn wait_done(app: &Router, slug: &str, codename: &str) -> Value {
    for _ in 0..200 {
        let (st, v) = send(
            app,
            get(&format!("/api/jobs/{slug}/applications/{codename}")),
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{v}");
        let pending = v["evaluations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["status"] == "pending" || e["status"] == "running");
        if !pending {
            return v;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("avaliações não terminaram");
}

// ------------------------------------------------------------------ testes

#[tokio::test]
async fn lists_engines_with_state() {
    let app = app().await;
    let (st, v) = send(&app, get("/api/engines")).await;
    assert_eq!(st, StatusCode::OK);
    let by_id = |id: &str| {
        v.as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == id)
            .cloned()
            .unwrap()
    };

    let h = by_id("heuristic-v1");
    assert_eq!(h["kind"], "heuristic");
    assert_eq!(h["execution"], "inline");
    assert_eq!(h["enabled"], true);
    assert_eq!(h["available"], true);
    assert_eq!(by_id("keywords-v1")["available"], true);
    assert_eq!(by_id("fake-llm")["kind"], "llm");
    assert_eq!(by_id("flaky-api")["kind"], "external_api");
    let k = by_id("needs-key");
    assert_eq!(k["available"], false);
    assert_eq!(k["unavailable_reason"], "defina FAKE_API_KEY");
}

#[tokio::test]
async fn apply_runs_every_active_engine() {
    let app = app().await;
    let slug = create_job(&app, None).await;
    let v = apply(&app, &slug, CV).await;

    // inline: já prontos na resposta do POST
    assert_eq!(v["primary_engine"], "heuristic-v1");
    assert_eq!(eval(&v, "heuristic-v1")["status"], "done");
    assert_eq!(eval(&v, "keywords-v1")["status"], "done");
    assert_eq!(eval(&v, "keywords-v1")["position"], 1);
    // nota fora da escala foi corrigida
    assert_eq!(eval(&v, "out-of-range")["score"], 100.0);
    // background: na fila
    assert!(["pending", "running"].contains(&eval(&v, "fake-llm")["status"].as_str().unwrap()));
    // indisponível: nem entra
    assert!(v["evaluations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["engine_id"] != "needs-key"));
    // o motor de ranking da vaga vem primeiro
    assert_eq!(v["evaluations"][0]["engine_id"], "heuristic-v1");

    let codename = v["codename"].as_str().unwrap();
    let done = wait_done(&app, &slug, codename).await;

    let llm = eval(&done, "fake-llm");
    assert_eq!(llm["status"], "done");
    assert_eq!(llm["engine_kind"], "llm");
    assert_eq!(llm["report"]["breakdown"][0]["key"], "fit");
    assert_eq!(llm["position"], 1);

    // falha transitória → retentativa → sucesso, sem erro residual
    let flaky = eval(&done, "flaky-api");
    assert_eq!(flaky["status"], "done", "{flaky}");
    assert_eq!(flaky["score"], 61.0);
    assert!(flaky["error"].is_null());

    // falha permanente → failed com mensagem, sem nota
    let broken = eval(&done, "broken-db");
    assert_eq!(broken["status"], "failed");
    assert_eq!(broken["error"], "resposta inválida");
    assert!(broken["score"].is_null());
}

#[tokio::test]
async fn ranking_per_engine() {
    let app = app().await;
    let slug = create_job(&app, None).await;
    let strong = apply(&app, &slug, CV).await;
    let weak_text = format!(
        "{}\n{}",
        "Vendedor de loja com experiência em atendimento ao público, \
        organização de vitrines e metas de vendas. Ensino médio completo.",
        "Atendimento. ".repeat(15)
    );
    let weak = apply(&app, &slug, &weak_text).await;
    for v in [&strong, &weak] {
        wait_done(&app, &slug, v["codename"].as_str().unwrap()).await;
    }

    for engine in ["heuristic-v1", "keywords-v1"] {
        let (st, r) = send(
            &app,
            get(&format!("/api/jobs/{slug}/ranking?engine={engine}")),
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{r}");
        let r = r.as_array().unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0]["engine_id"], engine);
        assert_eq!(
            r[0]["codename"], strong["codename"],
            "{engine}: forte deveria liderar"
        );
        assert!(r[0]["score"].as_f64() > r[1]["score"].as_f64());
    }
    // sem parâmetro = motor da vaga
    let (_, r) = send(&app, get(&format!("/api/jobs/{slug}/ranking"))).await;
    assert_eq!(r[0]["engine_id"], "heuristic-v1");
    // motor que falhou: ranking vazio (só entram avaliações done)
    let (_, r) = send(
        &app,
        get(&format!("/api/jobs/{slug}/ranking?engine=broken-db")),
    )
    .await;
    assert_eq!(r.as_array().unwrap().len(), 0);
    // motor inexistente
    let (st, _) = send(
        &app,
        get(&format!("/api/jobs/{slug}/ranking?engine=nao-existe")),
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn job_ranked_by_background_engine_falls_back_until_done() {
    let app = app().await;
    let (st, v) = send(
        &app,
        post_json(
            "/api/jobs",
            json!({"title": "X", "description": JOB_DESC, "ranking_engine": "needs-key"}),
        ),
    )
    .await;
    assert_eq!(
        st,
        StatusCode::BAD_REQUEST,
        "motor indisponível não pode ordenar vaga: {v}"
    );

    let slug = create_job(&app, Some("fake-llm")).await;
    let (_, job) = send(&app, get(&format!("/api/jobs/{slug}"))).await;
    assert_eq!(job["ranking_engine"], "fake-llm");
    assert!(job["engines"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["id"] != "needs-key"));

    let v = apply(&app, &slug, CV).await;
    // a LLM ainda não terminou: topo cai para o heuristic-v1
    assert_eq!(v["primary_engine"], "heuristic-v1");
    assert_eq!(v["evaluations"][0]["engine_id"], "fake-llm");

    let done = wait_done(&app, &slug, v["codename"].as_str().unwrap()).await;
    assert_eq!(done["primary_engine"], "fake-llm");
    assert_eq!(done["score"], eval(&done, "fake-llm")["score"]);

    // home: top 3 usa o motor da vaga
    let (_, jobs) = send(&app, get("/api/jobs")).await;
    let j = jobs
        .as_array()
        .unwrap()
        .iter()
        .find(|j| j["slug"] == slug.as_str())
        .unwrap();
    assert_eq!(j["ranking_engine"], "fake-llm");
    assert_eq!(j["top"][0]["score"], done["score"]);
}

#[tokio::test]
async fn deleting_resume_cascades_evaluations() {
    let pool = pokerum_api::db::connect("sqlite::memory:").await.unwrap();
    let app = app_with(pool.clone()).await;
    let slug = create_job(&app, None).await;
    let v = apply(&app, &slug, CV).await;
    let codename = v["codename"].as_str().unwrap();
    wait_done(&app, &slug, codename).await;

    let req = Request::delete(format!("/api/resumes/{codename}"))
        .header("x-owner-token", v["owner_token"].as_str().unwrap())
        .body(Body::empty())
        .unwrap();
    assert_eq!(send(&app, req).await.0, StatusCode::NO_CONTENT);
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM evaluations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
async fn disabled_engine_stops_evaluating_and_backfill_catches_up() {
    let pool = pokerum_api::db::connect("sqlite::memory:").await.unwrap();
    let runner = Runner::new(pool.clone(), registry(), 4).with_backoff(Duration::from_millis(10));
    runner.sync_catalog().await.unwrap();
    runner.set_enabled("fake-llm", false).await.unwrap();
    assert!(
        runner.set_enabled("heuristic-v1", false).await.is_err(),
        "padrão não pode ser desligado"
    );

    let state = pokerum_api::AppState::with_runner(pool.clone(), runner.clone())
        .await
        .unwrap();
    let app = pokerum_api::build_app(state);
    let slug = create_job(&app, None).await;
    let v = apply(&app, &slug, CV).await;
    let codename = v["codename"].as_str().unwrap();
    let done = wait_done(&app, &slug, codename).await;
    assert!(done["evaluations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["engine_id"] != "fake-llm"));

    // liga de novo e faz backfill: a candidatura antiga ganha a avaliação
    runner.set_enabled("fake-llm", true).await.unwrap();
    assert_eq!(runner.backfill("fake-llm").await.unwrap(), 1);
    assert_eq!(
        runner.backfill("fake-llm").await.unwrap(),
        0,
        "backfill é idempotente"
    );
    runner.resume_pending().await.unwrap();
    let done = wait_done(&app, &slug, codename).await;
    assert_eq!(eval(&done, "fake-llm")["status"], "done");

    // retry de falhas: volta para a fila e falha de novo (motor quebrado)
    assert_eq!(runner.retry_failed("broken-db").await.unwrap(), 1);
    runner.resume_pending().await.unwrap();
    let done = wait_done(&app, &slug, codename).await;
    assert_eq!(eval(&done, "broken-db")["status"], "failed");
}

#[tokio::test]
async fn migration_0002_backfills_existing_scores() {
    // Banco no estado da v1 (só 0001 aplicada, com uma candidatura antiga)...
    let dir = std::env::temp_dir().join(format!("pokerum-mig-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("v1.db");
    let _ = std::fs::remove_file(&path);
    let url = format!("sqlite://{}?mode=rwc", path.display());
    {
        let pool = sqlx::SqlitePool::connect(&url).await.unwrap();
        sqlx::raw_sql(include_str!("../migrations/0001_init.sql"))
            .execute(&pool)
            .await
            .unwrap();
        sqlx::raw_sql(
            "INSERT INTO jobs (id, slug, title, description, profile_json) VALUES (1, 'v', 'Vaga', 'desc', '{}');
             INSERT INTO resumes (id, codename, alias, anonymized_text, redactions_json, owner_token_hash)
                 VALUES (1, 'Pikachu', 'Ana', 'texto', '[]', 'h');
             INSERT INTO applications (job_id, resume_id, score, report_json, engine_version)
                 VALUES (1, 1, 42.5, '{\"total\":42.5,\"breakdown\":[],\"matched_keywords\":[],\"missing_keywords\":[],\"suggestions\":[],\"engine_version\":\"heuristic-v1\"}', 'heuristic-v1');",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;
    }
    // ...ao subir a versão nova, as migrations rodam e a nota antiga vira avaliação.
    let pool = pokerum_api::db::connect(&url).await.unwrap();
    let row: (String, String, f64) =
        sqlx::query_as("SELECT engine_id, status, score FROM evaluations WHERE application_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row, ("heuristic-v1".into(), "done".into(), 42.5));
    let re: String = sqlx::query_scalar("SELECT ranking_engine FROM jobs WHERE id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(re, "heuristic-v1");

    let app = app_with(pool).await;
    let (st, r) = send(&app, get("/api/jobs/v/ranking")).await;
    assert_eq!(st, StatusCode::OK, "{r}");
    assert_eq!(r[0]["codename"], "Pikachu");
    assert_eq!(r[0]["score"], 42.5);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn queue_worker_picks_up_cli_backfill_and_inline_engines_backfill_on_start() {
    let pool = pokerum_api::db::connect("sqlite::memory:").await.unwrap();
    let runner = Runner::new(pool.clone(), registry(), 4).with_backoff(Duration::from_millis(10));
    runner.sync_catalog().await.unwrap();
    runner.set_enabled("fake-llm", false).await.unwrap();
    runner.set_enabled("keywords-v1", false).await.unwrap();
    let state = pokerum_api::AppState::with_runner(pool.clone(), runner.clone())
        .await
        .unwrap();
    let app = pokerum_api::build_app(state);
    let slug = create_job(&app, None).await;
    let codename = apply(&app, &slug, CV).await["codename"]
        .as_str()
        .unwrap()
        .to_string();
    wait_done(&app, &slug, &codename).await;

    // motor inline religado: ao "reiniciar" (novo AppState), avalia o antigo sozinho
    runner.set_enabled("keywords-v1", true).await.unwrap();
    pokerum_api::AppState::with_runner(pool.clone(), runner.clone())
        .await
        .unwrap();
    let v = wait_done(&app, &slug, &codename).await;
    assert_eq!(eval(&v, "keywords-v1")["status"], "done");
    // ...mas motor de segundo plano (custo) não faz backfill sozinho
    runner.set_enabled("fake-llm", true).await.unwrap();
    pokerum_api::AppState::with_runner(pool.clone(), runner.clone())
        .await
        .unwrap();
    let v = wait_done(&app, &slug, &codename).await;
    assert!(v["evaluations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|e| e["engine_id"] != "fake-llm"));

    // backfill via "CLI" (outro Runner, como outro processo) + varredura da fila
    let worker = runner.start_queue_worker(Duration::from_millis(20));
    let cli = Runner::new(pool.clone(), registry(), 1);
    assert_eq!(cli.backfill("fake-llm").await.unwrap(), 1);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let v = wait_done(&app, &slug, &codename).await;
    assert_eq!(eval(&v, "fake-llm")["status"], "done");
    let attempts: i64 =
        sqlx::query_scalar("SELECT attempts FROM evaluations WHERE engine_id = 'fake-llm'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        attempts, 1,
        "a varredura não pode rodar a mesma avaliação duas vezes"
    );
    worker.abort();
}

#[tokio::test]
async fn engine_removed_from_code_is_hidden_and_jobs_fall_back() {
    // banco com o keywords-v1 registrado, com notas e ordenando uma vaga...
    let pool = pokerum_api::db::connect("sqlite::memory:").await.unwrap();
    let app = app_with(pool.clone()).await;
    let slug = create_job(&app, Some("keywords-v1")).await;
    let v = apply(&app, &slug, CV).await;
    let codename = v["codename"].as_str().unwrap().to_string();
    wait_done(&app, &slug, &codename).await;
    assert_eq!(eval(&v, "keywords-v1")["status"], "done");

    // ...o servidor sobe com uma versão que não registra mais o keywords-v1
    let state = pokerum_api::AppState::new(pool.clone()).await.unwrap();
    let app = pokerum_api::build_app(state);

    let (_, engines) = send(&app, get("/api/engines")).await;
    let ids: Vec<_> = engines
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["heuristic-v1"]);

    let (_, job) = send(&app, get(&format!("/api/jobs/{slug}"))).await;
    assert_eq!(
        job["ranking_engine"], "heuristic-v1",
        "vaga cai para o motor padrão"
    );
    assert_eq!(job["engines"].as_array().unwrap().len(), 1);

    let (_, d) = send(
        &app,
        get(&format!("/api/jobs/{slug}/applications/{codename}")),
    )
    .await;
    assert_eq!(d["primary_engine"], "heuristic-v1");
    let evs: Vec<_> = d["evaluations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["engine_id"].as_str().unwrap())
        .collect();
    assert_eq!(
        evs,
        ["heuristic-v1"],
        "notas de motor desligado não aparecem"
    );

    let (st, _) = send(
        &app,
        get(&format!("/api/jobs/{slug}/ranking?engine=keywords-v1")),
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    let (_, list) = send(&app, get("/api/jobs")).await;
    assert_eq!(list[0]["ranking_engine"], "heuristic-v1");
    assert_eq!(list[0]["top"].as_array().unwrap().len(), 1);

    // as notas antigas continuam guardadas: voltam se o motor for religado
    let n: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM evaluations WHERE engine_id = 'keywords-v1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(n, 1);
}
