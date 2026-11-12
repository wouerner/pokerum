//! `--seed`: popula o banco com vagas e currículos fictícios se estiver vazio.

use crate::models::CreateJob;
use crate::service;

const JOB_RUST: &str = "Buscamos uma pessoa Desenvolvedora Backend Rust Sênior para atuar no time de \
plataforma de pagamentos.\n\nResponsabilidades:\n- Projetar e construir APIs REST e gRPC de alta \
performance em Rust (tokio, axum)\n- Modelar dados em PostgreSQL e otimizar consultas\n- Trabalhar com \
mensageria (Kafka) e arquitetura orientada a eventos\n- Garantir observabilidade (Prometheus, Grafana, \
OpenTelemetry)\n\nRequisitos:\n- 5+ anos de experiência com desenvolvimento backend\n- Experiência \
sólida com Rust em produção\n- Docker, Kubernetes, CI/CD\n- AWS ou GCP\n- Inglês intermediário/avançado\n\n\
Diferenciais: Go, Redis, sistemas distribuídos, contribuição open source.";

const JOB_DATA: &str = "Procuramos Engenheira(o) de Dados Pleno para construir e manter pipelines \
de dados.\n\nAtividades:\n- Desenvolver pipelines ETL/ELT em Python e SQL\n- Orquestrar fluxos com \
Airflow\n- Processamento distribuído com Spark\n- Modelagem dimensional em data warehouse (BigQuery ou \
Snowflake)\n- Garantir qualidade de dados com dbt e testes\n\nRequisitos:\n- 3+ anos de experiência \
com engenharia de dados\n- Python, SQL avançado, Spark, Airflow\n- Cloud (GCP ou AWS)\n- Graduação em \
Ciência da Computação, Engenharia ou áreas afins\n- Inglês técnico para leitura";

const CV_RUST_1: &str = "Camila Ferreira Rocha\nEmail: camila.rocha@example.com | Telefone: (11) 98765-4321\n\
São Paulo - SP | linkedin.com/in/camila-rocha-exemplo\n\nResumo\nDesenvolvedora Backend Sênior com 8 anos \
de experiência construindo sistemas distribuídos de alta disponibilidade.\n\nExperiência\n\
Desenvolvedora Backend Sênior - FinPag (2020 - atual)\n- Desenvolvi serviços em Rust com tokio e axum \
processando 20 mil transações por segundo\n- Modelagem e tuning de PostgreSQL, uso de Redis como cache\n\
- Integração orientada a eventos com Kafka\n- Observabilidade com Prometheus, Grafana e OpenTelemetry\n\n\
Desenvolvedora Backend - LojaNet (2016 - 2020)\n- APIs REST em Go e Python\n- Deploy com Docker e \
Kubernetes na AWS, pipelines de CI/CD\n\nFormação\nBacharelado em Ciência da Computação - USP (2015)\n\n\
Habilidades\nRust, Go, Python, PostgreSQL, Redis, Kafka, Docker, Kubernetes, AWS, gRPC, CI/CD\n\n\
Idiomas\nInglês avançado";

const CV_RUST_2: &str = "Rafael Nogueira Lima\nrafael.lima@example.com - (21) 99876-1234\n\n\
Objetivo: atuar como desenvolvedor backend.\n\nExperiência Profissional\nDesenvolvedor Pleno - \
TechSul (2019 - 2024)\n- Desenvolvimento de APIs em Java com Spring Boot\n- Banco de dados MySQL e \
PostgreSQL\n- Estudos pessoais em Rust (projetos de linha de comando)\n- Uso de Docker no ambiente de \
desenvolvimento\n\nEstagiário - Agência Web (2018 - 2019)\n- Manutenção de sites em PHP\n\nFormação\n\
Tecnólogo em Análise e Desenvolvimento de Sistemas (2019)\n\nCompetências\nJava, Spring, SQL, Docker, \
Git, Rust (básico)\n\nIdiomas\nInglês intermediário";

const CV_DATA_1: &str = "Beatriz Almeida Santos\nbeatriz.santos@example.com | +55 31 98888-7777\n\
Belo Horizonte, MG\n\nPerfil\nEngenheira de Dados Pleno com 4 anos de experiência em pipelines de dados \
em nuvem.\n\nExperiência\nEngenheira de Dados - DataMinas (2021 - atual)\n- Pipelines ETL em Python e SQL \
orquestrados com Airflow\n- Processamento distribuído com Spark no GCP (Dataproc)\n- Modelagem dimensional \
no BigQuery e transformações com dbt\n- Testes de qualidade de dados e monitoramento\n\nAnalista de Dados - \
Varejo BH (2020 - 2021)\n- SQL avançado, dashboards e automações em Python\n\nFormação\nEngenharia de \
Computação - UFMG (2019)\nCertificação Google Cloud Professional Data Engineer\n\nSkills\nPython, SQL, \
Spark, Airflow, dbt, BigQuery, GCP, Git, Docker\n\nIdiomas\nInglês avançado";

const CV_DATA_2: &str = "Lucas Pereira Costa\nlucas.costa@example.com\n\nResumo\nAnalista de BI \
buscando transição para engenharia de dados.\n\nExperiência\nAnalista de BI - Logística XYZ (2021 - 2024)\n\
- Criação de relatórios no Power BI e consultas SQL\n- Automação de planilhas com Python (pandas)\n- \
Participação em projeto de migração para Snowflake\n\nFormação\nAdministração - Universidade Estadual (2020)\n\
Curso livre de Apache Spark (2023)\n\nHabilidades\nSQL, Power BI, Excel, Python, pandas, Snowflake";

/// (título, empresa, descrição, [(currículo, nome real)])
type SampleJob = (
    &'static str,
    Option<&'static str>,
    &'static str,
    &'static [(&'static str, &'static str)],
);

pub async fn run(state: &crate::AppState) -> anyhow::Result<()> {
    let pool = &state.pool;
    let jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM jobs")
        .fetch_one(pool)
        .await?;
    if jobs > 0 {
        tracing::info!(jobs, "seed ignorado: banco já possui vagas");
        return Ok(());
    }

    let samples: [SampleJob; 2] = [
        (
            "Desenvolvedor Backend Rust Sênior",
            Some("PagaFácil (fictícia)"),
            JOB_RUST,
            &[
                (CV_RUST_1, "Camila Ferreira Rocha"),
                (CV_RUST_2, "Rafael Nogueira Lima"),
            ],
        ),
        (
            "Engenheira(o) de Dados Pleno",
            Some("DadosBR (fictícia)"),
            JOB_DATA,
            &[
                (CV_DATA_1, "Beatriz Almeida Santos"),
                (CV_DATA_2, "Lucas Pereira Costa"),
            ],
        ),
    ];

    for (title, company, description, cvs) in samples {
        let job = service::create_job(
            pool,
            &state.runner,
            CreateJob {
                title: title.into(),
                company: company.map(Into::into),
                description: description.into(),
                ranking_engine: None,
            },
        )
        .await
        .map_err(|e| anyhow::anyhow!("criando vaga {title}: {e}"))?;
        let row = service::job_by_slug(pool, &job.slug)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        for (cv, name) in cvs.iter() {
            let r = service::apply(pool, &state.runner, &row, cv, Some(name))
                .await
                .map_err(|e| anyhow::anyhow!("aplicando em {title}: {e}"))?;
            tracing::info!(job = %job.slug, codename = %r.codename, score = r.score, "seed: candidatura");
        }
    }
    tracing::info!("seed concluído");
    Ok(())
}
