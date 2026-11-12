//! `heuristic-v1`: o motor original do Pokerum (6 critérios ponderados, ver CONTRACT.md).

use async_trait::async_trait;
use pokerum_core::scoring::{score, ScoreReport};

use super::{EngineError, EngineInfo, EngineKind, EvaluationInput, Execution, ScoringEngine};

pub const ID: &str = "heuristic-v1";

pub struct HeuristicEngine;

#[async_trait]
impl ScoringEngine for HeuristicEngine {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: ID,
            name: "Heurístico v1",
            kind: EngineKind::Heuristic,
            description: "Seis critérios ponderados: skills, similaridade, experiência, senioridade, formação/idiomas e formato ATS.",
            execution: Execution::Inline,
        }
    }

    async fn evaluate(&self, input: &EvaluationInput) -> Result<ScoreReport, EngineError> {
        Ok(score(&input.job_profile, &input.resume_text))
    }
}
