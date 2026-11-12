//! Motores de avaliação plugáveis.
//!
//! Um motor recebe a vaga + o currículo **já anonimizado** e devolve um [`ScoreReport`].
//! Pode ser uma heurística local, uma LLM, uma API externa, uma consulta a banco etc.
//! Cada candidatura é avaliada por todos os motores ativos e o resultado fica na
//! tabela `evaluations`. Ver `README.md` nesta pasta para adicionar um motor novo.

pub mod heuristic;
/// Exemplo de motor (não registrado em `builtin`); ver `README.md`.
pub mod keywords;
pub mod runner;

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use pokerum_core::scoring::{JobProfile, ScoreReport};
use serde::{Deserialize, Serialize};

pub use runner::Runner;

/// Família do motor (só informativo/UI; não muda como ele é executado).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    Heuristic,
    Llm,
    ExternalApi,
    Database,
}

impl EngineKind {
    pub fn as_str(self) -> &'static str {
        match self {
            EngineKind::Heuristic => "heuristic",
            EngineKind::Llm => "llm",
            EngineKind::ExternalApi => "external_api",
            EngineKind::Database => "database",
        }
    }
}

/// Onde o motor roda.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Execution {
    /// Rápido e local: roda dentro do POST /apply e o resultado já volta na resposta.
    Inline,
    /// Lento/sujeito a rede (LLM, API): entra na fila e roda em segundo plano.
    Background,
}

impl Execution {
    pub fn as_str(self) -> &'static str {
        match self {
            Execution::Inline => "inline",
            Execution::Background => "background",
        }
    }
}

/// Metadados estáticos do motor.
#[derive(Debug, Clone)]
pub struct EngineInfo {
    /// Identificador estável e versionado (ex.: `heuristic-v1`, `claude-v1`).
    /// Mudou o algoritmo de forma que as notas não são comparáveis? Crie um id novo.
    pub id: &'static str,
    pub name: &'static str,
    pub kind: EngineKind,
    pub description: &'static str,
    pub execution: Execution,
}

/// Tudo o que um motor pode usar para avaliar. NUNCA contém dados pessoais:
/// o texto do currículo já passou pela anonimização.
#[derive(Debug, Clone)]
pub struct EvaluationInput {
    pub job_title: String,
    pub job_description: String,
    /// Análise da vaga feita pelo core (skills, anos, senioridade...).
    pub job_profile: JobProfile,
    pub resume_codename: String,
    pub resume_text: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    /// Falha passageira (timeout, 429, 5xx): o executor tenta de novo.
    #[error("falha temporária: {0}")]
    Transient(String),
    /// Falha definitiva (resposta inválida, configuração errada): não adianta repetir.
    #[error("{0}")]
    Permanent(String),
}

impl EngineError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, EngineError::Transient(_))
    }
}

#[async_trait]
pub trait ScoringEngine: Send + Sync + 'static {
    fn info(&self) -> EngineInfo;

    /// `Err(motivo)` quando falta configuração (ex.: API key ausente). Motores
    /// indisponíveis ficam registrados mas não avaliam nada.
    fn availability(&self) -> Result<(), String> {
        Ok(())
    }

    /// Tempo máximo de uma tentativa.
    fn timeout(&self) -> Duration {
        match self.info().execution {
            Execution::Inline => Duration::from_secs(10),
            Execution::Background => Duration::from_secs(120),
        }
    }

    /// Nº máximo de tentativas para erros [`EngineError::Transient`].
    fn max_attempts(&self) -> u32 {
        match self.info().execution {
            Execution::Inline => 1,
            Execution::Background => 3,
        }
    }

    /// Avalia o currículo contra a vaga. `ScoreReport.total` deve estar em 0..=100;
    /// `breakdown[].key` é livre (cada motor tem seus critérios).
    async fn evaluate(&self, input: &EvaluationInput) -> Result<ScoreReport, EngineError>;
}

/// Motor padrão (sempre inline, sempre disponível): usado como fallback.
pub const DEFAULT_ENGINE: &str = heuristic::ID;

/// Conjunto de motores compilados no binário, por id.
#[derive(Clone, Default)]
pub struct EngineRegistry {
    engines: BTreeMap<&'static str, Arc<dyn ScoringEngine>>,
}

impl EngineRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registro com os motores embutidos. Adicione motores novos aqui.
    pub fn builtin() -> Self {
        let mut r = Self::new();
        r.register(heuristic::HeuristicEngine);
        // `keywords-v1` ("ATS clássico") fica no código como exemplo de motor, mas não é
        // registrado: por enquanto o Pokerum usa só o heuristic-v1.
        // Ex. futuro:
        // r.register(llm::ClaudeEngine::from_env());
        // r.register(external::WebhookEngine::from_env());
        r
    }

    pub fn register(&mut self, engine: impl ScoringEngine) -> &mut Self {
        let id = engine.info().id;
        if self.engines.insert(id, Arc::new(engine)).is_some() {
            panic!("motor de avaliação duplicado: {id}");
        }
        self
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn ScoringEngine>> {
        self.engines.get(id).cloned()
    }

    pub fn all(&self) -> impl Iterator<Item = &Arc<dyn ScoringEngine>> {
        self.engines.values()
    }
}

/// Garante que o relatório de um motor é utilizável (nota finita em 0..=100).
pub(crate) fn sanitize_report(mut report: ScoreReport) -> Result<ScoreReport, EngineError> {
    if !report.total.is_finite() {
        return Err(EngineError::Permanent(
            "motor devolveu nota inválida".into(),
        ));
    }
    report.total = (report.total.clamp(0.0, 100.0) * 10.0).round() / 10.0;
    for c in &mut report.breakdown {
        if !c.score.is_finite() {
            c.score = 0.0;
        }
        c.score = c.score.clamp(0.0, 100.0);
    }
    Ok(report)
}
