use std::{net::SocketAddr, path::PathBuf};

use anyhow::Context;
use tracing_subscriber::EnvFilter;

fn resolve_web_dist() -> Option<PathBuf> {
    let candidates: Vec<PathBuf> = match std::env::var("POKERUM_WEB_DIST") {
        Ok(p) if !p.trim().is_empty() => vec![PathBuf::from(p)],
        _ => vec![
            PathBuf::from("web/dist"),
            PathBuf::from("../web/dist"),
            PathBuf::from("../../web/dist"),
        ],
    };
    candidates
        .into_iter()
        .find(|p| p.join("index.html").is_file())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,sqlx=warn")),
        )
        .init();

    let seed = std::env::args().skip(1).any(|a| a == "--seed")
        || matches!(
            std::env::var("POKERUM_SEED").as_deref(),
            Ok("1") | Ok("true")
        );
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://pokerum.db?mode=rwc".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let pool = pokerum_api::db::connect(&database_url)
        .await
        .with_context(|| format!("abrindo banco {database_url}"))?;

    let concurrency: usize = std::env::var("POKERUM_ENGINE_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4);
    let registry = pokerum_api::engines::EngineRegistry::builtin();

    // Subcomando de administração: `pokerum_api engines ...`
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("engines") {
        let runner = pokerum_api::engines::Runner::new(pool.clone(), registry, concurrency);
        runner.sync_catalog().await?;
        return engines_cli(&runner, &args[1..]).await;
    }

    let state = pokerum_api::AppState::with_registry(pool, registry, concurrency)
        .await
        .context("iniciando motores de avaliação")?;
    if seed {
        pokerum_api::seed::run(&state).await.context("seed")?;
    }
    let poll_secs: u64 = std::env::var("POKERUM_QUEUE_POLL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(15);
    state
        .runner
        .start_queue_worker(std::time::Duration::from_secs(poll_secs.max(1)));

    let mut app = pokerum_api::build_app(state);
    match resolve_web_dist() {
        Some(dist) => {
            tracing::info!(dir = %dist.display(), "servindo SPA estático");
            let ga_id = pokerum_api::analytics::ga_id_from_env();
            if let Some(id) = &ga_id {
                tracing::info!(id, "Google Analytics ligado (Consent Mode)");
            }
            app = pokerum_api::with_static(app, &dist, ga_id.as_deref());
        }
        None => tracing::info!("web/dist não encontrado; servindo apenas /api"),
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "pokerum_api ouvindo");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    tracing::info!("pokerum_api encerrado");
    Ok(())
}

/// Ctrl+C (local) ou SIGTERM (`docker stop`, deploy no Coolify). No container o processo é o
/// PID 1, que ignora SIGTERM se não houver tratador: sem isto cada deploy esperaria o SIGKILL.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(e) => {
                tracing::warn!(error = %e, "sem tratador de SIGTERM");
                std::future::pending::<()>().await
            }
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }
    tracing::info!(
        "sinal recebido; encerrando (avaliações pendentes são retomadas no próximo start)"
    );
}

const ENGINES_HELP: &str = "uso: pokerum_api engines <comando>
  list               lista os motores e seu estado
  enable  <id>       liga o motor (passa a avaliar candidaturas novas)
  disable <id>       desliga o motor
  backfill <id>      enfileira o motor para todas as candidaturas sem avaliação dele
  retry <id>         recoloca na fila as avaliações que falharam desse motor
(backfill/retry só enfileiram: o servidor processa a fila quando estiver rodando)";

async fn engines_cli(runner: &pokerum_api::engines::Runner, args: &[String]) -> anyhow::Result<()> {
    let id = args.get(1).map(String::as_str);
    let err = |e: pokerum_api::error::AppError| anyhow::anyhow!("{e}");
    match (args.first().map(String::as_str), id) {
        (Some("list") | None, _) => {
            println!(
                "{:<16} {:<13} {:<11} {:<8} {:<10} nome",
                "id", "tipo", "execução", "ligado", "config."
            );
            for e in runner.engines().await.map_err(err)? {
                println!(
                    "{:<16} {:<13} {:<11} {:<8} {:<10} {}{}",
                    e.id,
                    e.kind.as_str(),
                    e.execution.as_str(),
                    if e.enabled { "sim" } else { "não" },
                    if e.available { "ok" } else { "faltando" },
                    e.name,
                    e.unavailable_reason
                        .map(|r| format!(" ({r})"))
                        .unwrap_or_default(),
                );
            }
        }
        (Some("enable"), Some(id)) => {
            runner.set_enabled(id, true).await.map_err(err)?;
            println!("motor {id} ligado");
        }
        (Some("disable"), Some(id)) => {
            runner.set_enabled(id, false).await.map_err(err)?;
            println!("motor {id} desligado");
        }
        (Some("backfill"), Some(id)) => {
            let n = runner.backfill(id).await.map_err(err)?;
            println!("{n} avaliações enfileiradas para {id}");
        }
        (Some("retry"), Some(id)) => {
            let n = runner.retry_failed(id).await.map_err(err)?;
            println!("{n} avaliações de {id} recolocadas na fila");
        }
        _ => {
            eprintln!("{ENGINES_HELP}");
            std::process::exit(2);
        }
    }
    Ok(())
}
