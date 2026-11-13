use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

async fn app() -> Router {
    let pool = pokerum_api::db::connect("sqlite::memory:").await.unwrap();
    pokerum_api::build_app(pokerum_api::AppState::new(pool).await.unwrap())
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()))
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

const BOUNDARY: &str = "XPOKERUMBOUNDARYX";

fn multipart(uri: &str, parts: &[(&str, Option<&str>, &str)]) -> Request<Body> {
    let mut body = String::new();
    for (name, filename, content) in parts {
        body.push_str(&format!("--{BOUNDARY}\r\n"));
        match filename {
            Some(f) => body.push_str(&format!(
                "Content-Disposition: form-data; name=\"{name}\"; filename=\"{f}\"\r\nContent-Type: text/plain\r\n\r\n"
            )),
            None => body.push_str(&format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n")),
        }
        body.push_str(content);
        body.push_str("\r\n");
    }
    body.push_str(&format!("--{BOUNDARY}--\r\n"));
    Request::post(uri)
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={BOUNDARY}"),
        )
        .body(Body::from(body))
        .unwrap()
}

const JOB_DESC: &str =
    "Vaga para desenvolvedor backend Rust sênior. Requisitos: Rust, tokio, axum, \
PostgreSQL, Docker, Kubernetes, AWS, Kafka. 5 anos de experiência. Inglês avançado.";

const STRONG_CV: &str = "Joana Martins Silva\njoana.silva@example.com (11) 91234-5678\n\nResumo\n\
Desenvolvedora backend sênior com 7 anos de experiência.\n\nExperiência\nDesenvolvedora Backend Sênior \
- Empresa X (2018 - atual)\n- Serviços em Rust com tokio e axum\n- PostgreSQL, Kafka, Docker, Kubernetes \
na AWS\n\nFormação\nCiência da Computação (2017)\n\nHabilidades\nRust, tokio, axum, PostgreSQL, Docker, \
Kubernetes, AWS, Kafka\n\nIdiomas\nInglês avançado";

const WEAK_CV: &str = "Pedro Henrique Souza\npedro@example.com\n\nResumo\nProfissional de vendas com \
experiência em atendimento ao cliente, negociação e gestão de carteira de clientes no varejo de moda.\n\n\
Experiência\nVendedor - Loja de Roupas (2020 - 2023)\n- Atendimento ao público e organização de vitrines\n\
- Metas mensais de vendas batidas com frequência\n\nFormação\nEnsino médio completo";

async fn create_job(app: &Router) -> String {
    let (st, v) = send(
        app,
        post_json("/api/jobs", json!({"title": "Desenvolvedor Backend Rust Sênior", "company": "ACME", "description": JOB_DESC})),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "{v}");
    v["slug"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn health() {
    let app = app().await;
    let (st, v) = send(&app, get("/api/health")).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v, json!({"ok": true}));
}

#[tokio::test]
async fn create_and_get_job() {
    let app = app().await;
    let slug = create_job(&app).await;
    assert!(
        slug.starts_with("desenvolvedor-backend-rust-senior-"),
        "{slug}"
    );

    let (st, v) = send(&app, get(&format!("/api/jobs/{slug}"))).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v["title"], "Desenvolvedor Backend Rust Sênior");
    assert_eq!(v["company"], "ACME");
    assert!(v["keywords"].is_array());

    let (st, v) = send(&app, get("/api/jobs")).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v[0]["applicants"], 0);

    let (st, v) = send(&app, get("/api/jobs/nao-existe")).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    assert!(v["error"].is_string());

    let (st, v) = send(
        &app,
        post_json("/api/jobs", json!({"title": "", "description": "x"})),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST);
    assert!(v["error"].is_string());
}

#[tokio::test]
async fn apply_with_text_and_ranking_and_delete() {
    let app = app().await;
    let slug = create_job(&app).await;

    // texto curto demais
    let (st, v) = send(
        &app,
        multipart(
            &format!("/api/jobs/{slug}/apply"),
            &[("text", None, "curto")],
        ),
    )
    .await;
    assert_eq!(st, StatusCode::BAD_REQUEST, "{v}");

    // candidato fraco primeiro (campo text + name)
    let (st, weak) = send(
        &app,
        multipart(
            &format!("/api/jobs/{slug}/apply"),
            &[
                ("name", None, "Pedro Henrique Souza"),
                ("text", None, WEAK_CV),
            ],
        ),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "{weak}");
    assert_eq!(weak["owner_token"].as_str().unwrap().len(), 64);
    assert_eq!(weak["position"], 1);
    assert_eq!(weak["total"], 1);
    let anon = weak["anonymized_text"].as_str().unwrap();
    assert!(!anon.contains("pedro@example.com"), "email vazou: {anon}");

    // candidato forte via arquivo .txt
    let (st, strong) = send(
        &app,
        multipart(
            &format!("/api/jobs/{slug}/apply"),
            &[
                ("name", None, "Joana Martins Silva"),
                ("file", Some("cv.txt"), STRONG_CV),
            ],
        ),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "{strong}");
    assert_eq!(strong["total"], 2);
    assert_ne!(strong["codename"], weak["codename"]);
    let anon = strong["anonymized_text"].as_str().unwrap();
    assert!(!anon.contains("joana.silva@example.com"));
    assert!(!anon.contains("Joana Martins Silva"));
    assert!(strong["score"].as_f64().unwrap() > weak["score"].as_f64().unwrap());
    assert_eq!(strong["position"], 1);

    // ranking ordenado
    let (st, rank) = send(&app, get(&format!("/api/jobs/{slug}/ranking"))).await;
    assert_eq!(st, StatusCode::OK);
    let rank = rank.as_array().unwrap().clone();
    assert_eq!(rank.len(), 2);
    assert_eq!(rank[0]["position"], 1);
    assert_eq!(rank[0]["codename"], strong["codename"]);
    assert_eq!(rank[1]["position"], 2);
    assert_eq!(rank[1]["codename"], weak["codename"]);
    assert!(rank[0]["score"].as_f64().unwrap() >= rank[1]["score"].as_f64().unwrap());
    assert!(!rank[0]["breakdown"].as_array().unwrap().is_empty());

    // nada de PII no banco: o nome real não aparece em lugar nenhum da API
    let codename = strong["codename"].as_str().unwrap();
    let (st, resume) = send(&app, get(&format!("/api/resumes/{codename}"))).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(resume["applications"][0]["job_slug"], slug);
    assert_eq!(resume["applications"][0]["position"], 1);
    assert_eq!(resume["applications"][0]["total"], 2);

    let (st, detail) = send(
        &app,
        get(&format!("/api/jobs/{slug}/applications/{codename}")),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(detail["position"], 1);
    assert!(detail["report"]["breakdown"].is_array());

    let (_, jobs) = send(&app, get("/api/jobs")).await;
    assert_eq!(jobs[0]["applicants"], 2);

    // delete com token errado / sem token
    let del = |tok: Option<&str>| {
        let mut b = Request::delete(format!("/api/resumes/{codename}"));
        if let Some(t) = tok {
            b = b.header("X-Owner-Token", t);
        }
        b.body(Body::empty()).unwrap()
    };
    let (st, _) = send(&app, del(None)).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    let (st, v) = send(&app, del(Some("deadbeef"))).await;
    assert_eq!(st, StatusCode::FORBIDDEN);
    assert!(v["error"].is_string());

    // token certo
    let (st, _) = send(&app, del(strong["owner_token"].as_str())).await;
    assert_eq!(st, StatusCode::NO_CONTENT);
    let (st, _) = send(&app, get(&format!("/api/resumes/{codename}"))).await;
    assert_eq!(st, StatusCode::NOT_FOUND);

    let (_, rank) = send(&app, get(&format!("/api/jobs/{slug}/ranking"))).await;
    assert_eq!(rank.as_array().unwrap().len(), 1);
    assert_eq!(rank[0]["position"], 1);
    assert_eq!(rank[0]["codename"], weak["codename"]);
}

#[tokio::test]
async fn apply_rejects_unsupported_file() {
    let app = app().await;
    let slug = create_job(&app).await;
    let (st, v) = send(
        &app,
        multipart(
            &format!("/api/jobs/{slug}/apply"),
            &[("file", Some("cv.docx"), "PK lixo")],
        ),
    )
    .await;
    assert_eq!(st, StatusCode::UNSUPPORTED_MEDIA_TYPE, "{v}");
    assert!(v["error"].is_string());
}

#[tokio::test]
async fn apply_accepts_json_body() {
    let app = app().await;
    let slug = create_job(&app).await;
    let (st, v) = send(
        &app,
        post_json(
            &format!("/api/jobs/{slug}/apply"),
            json!({"text": STRONG_CV}),
        ),
    )
    .await;
    assert_eq!(st, StatusCode::CREATED, "{v}");
    assert_eq!(v["position"], 1);
}

#[tokio::test]
async fn seed_populates_empty_db_once() {
    let pool = pokerum_api::db::connect("sqlite::memory:").await.unwrap();
    let state = pokerum_api::AppState::new(pool).await.unwrap();
    pokerum_api::seed::run(&state).await.unwrap();
    pokerum_api::seed::run(&state).await.unwrap();
    let app = pokerum_api::build_app(state);
    let (_, jobs) = send(&app, get("/api/jobs")).await;
    let jobs = jobs.as_array().unwrap();
    assert_eq!(jobs.len(), 2);
    assert!(jobs.iter().all(|j| j["applicants"] == 2));
}

#[tokio::test]
async fn job_list_includes_top3() {
    let app = app().await;
    let slug = create_job(&app).await;

    // vaga sem candidatos → top vazio
    let (_, list) = send(&app, get("/api/jobs")).await;
    assert_eq!(list[0]["top"].as_array().unwrap().len(), 0);

    // 4 candidaturas: 2 fortes, 2 fracas
    for cv in [WEAK_CV, STRONG_CV, WEAK_CV, STRONG_CV] {
        let (st, v) = send(
            &app,
            multipart(&format!("/api/jobs/{slug}/apply"), &[("text", None, cv)]),
        )
        .await;
        assert_eq!(st, StatusCode::CREATED, "{v}");
    }

    let (st, list) = send(&app, get("/api/jobs")).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(list[0]["applicants"], 4);
    let top = list[0]["top"].as_array().unwrap();
    assert_eq!(top.len(), 3, "{list}");

    // mesma ordem do ranking completo
    let (_, ranking) = send(&app, get(&format!("/api/jobs/{slug}/ranking"))).await;
    for (i, c) in top.iter().enumerate() {
        assert_eq!(c["position"], i as i64 + 1);
        assert_eq!(c["codename"], ranking[i]["codename"]);
        assert_eq!(c["score"], ranking[i]["score"]);
    }
    assert!(top[0]["score"].as_f64().unwrap() >= top[2]["score"].as_f64().unwrap());
}

/// SPA com e sem Google Analytics: a tag só aparece com ID, em `/`, `/index.html` e no fallback
/// de rota do Vue; os demais arquivos saem do disco sem mudança.
#[tokio::test]
async fn spa_injects_analytics_only_when_configured() {
    let dir = std::env::temp_dir().join(format!("pokerum-dist-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(
        dir.join("index.html"),
        "<html><head><title>P</title></head><body></body></html>",
    )
    .unwrap();
    std::fs::write(dir.join("assets/app.js"), "console.log(1)").unwrap();

    let plain = pokerum_api::with_static(app().await, &dir, None);
    let (s, v) = send(&plain, get("/vagas/qualquer")).await;
    assert_eq!(s, StatusCode::OK);
    assert!(!v.as_str().unwrap().contains("gtag"));

    let ga = pokerum_api::with_static(app().await, &dir, Some("G-TEST1234"));
    for uri in ["/", "/index.html", "/vagas/qualquer"] {
        let (s, v) = send(&ga, get(uri)).await;
        assert_eq!(s, StatusCode::OK, "{uri}");
        let html = v.as_str().unwrap();
        assert!(html.contains("gtag/js?id=G-TEST1234"), "{uri}");
        assert!(html.find("gtag/js").unwrap() < html.find("</head>").unwrap());
    }
    let (s, v) = send(&ga, get("/assets/app.js")).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(v.as_str().unwrap(), "console.log(1)");
    let (s, _) = send(&ga, get("/api/health")).await;
    assert_eq!(s, StatusCode::OK);

    std::fs::remove_dir_all(&dir).ok();
}
