//! `keywords-v1`: "ATS clássico". Só olha cobertura de palavras-chave — como os filtros
//! de triagem antigos, que não entendem contexto, sinônimos fora do dicionário nem experiência.

use std::collections::HashSet;

use async_trait::async_trait;
use pokerum_core::scoring::{skills, CriterionScore, ScoreReport};
use pokerum_core::text::{stem, tokens, NgramSet};

use super::{EngineError, EngineInfo, EngineKind, EvaluationInput, Execution, ScoringEngine};

pub const ID: &str = "keywords-v1";

const W_REQUIRED: f64 = 0.60;
const W_NICE: f64 = 0.15;
const W_TERMS: f64 = 0.25;

pub struct KeywordsEngine;

fn pct(hit: usize, of: usize) -> f64 {
    if of == 0 {
        100.0
    } else {
        hit as f64 / of as f64 * 100.0
    }
}

#[async_trait]
impl ScoringEngine for KeywordsEngine {
    fn info(&self) -> EngineInfo {
        EngineInfo {
            id: ID,
            name: "ATS clássico",
            kind: EngineKind::Heuristic,
            description: "Filtro por palavras-chave, como os ATS tradicionais: conta quantas skills e termos da vaga aparecem no currículo.",
            execution: Execution::Inline,
        }
    }

    async fn evaluate(&self, input: &EvaluationInput) -> Result<ScoreReport, EngineError> {
        let job = &input.job_profile;
        let raw = &input.resume_text;
        let toks = tokens(raw);
        let ngrams = NgramSet::from_tokens(&toks);
        let stems: HashSet<String> = toks.iter().map(|t| stem(t)).collect();

        let mut matched = Vec::new();
        let mut missing = Vec::new();
        let (mut req_hit, mut req_total, mut nice_hit, mut nice_total) = (0, 0, 0, 0);
        for s in &job.skills {
            let found = skills::by_name(&s.name)
                .map(|def| skills::present(def, &ngrams, raw))
                .unwrap_or_else(|| ngrams.has_phrase(&s.name));
            if s.required {
                req_total += 1;
                req_hit += found as usize;
            } else {
                nice_total += 1;
                nice_hit += found as usize;
            }
            if found {
                matched.push(s.name.clone());
            } else {
                missing.push(s.name.clone());
            }
        }

        let terms: Vec<_> = job.important_terms.iter().take(20).collect();
        let term_hit = terms.iter().filter(|t| stems.contains(&t.stem)).count();
        let missing_terms: Vec<String> = terms
            .iter()
            .filter(|t| !stems.contains(&t.stem))
            .map(|t| t.term.clone())
            .collect();

        let breakdown = vec![
            CriterionScore {
                key: "required_keywords".into(),
                label: "Skills obrigatórias".into(),
                weight: W_REQUIRED,
                score: pct(req_hit, req_total),
                detail: format!("{req_hit} de {req_total} encontradas"),
            },
            CriterionScore {
                key: "nice_keywords".into(),
                label: "Skills desejáveis".into(),
                weight: W_NICE,
                score: pct(nice_hit, nice_total),
                detail: format!("{nice_hit} de {nice_total} encontradas"),
            },
            CriterionScore {
                key: "job_terms".into(),
                label: "Termos da vaga".into(),
                weight: W_TERMS,
                score: pct(term_hit, terms.len()),
                detail: format!("{term_hit} de {} termos importantes", terms.len()),
            },
        ];
        let total: f64 = breakdown.iter().map(|c| c.weight * c.score).sum();

        let mut suggestions = Vec::new();
        if !missing.is_empty() {
            let list: Vec<_> = missing.iter().take(6).map(|m| format!("'{m}'")).collect();
            suggestions.push(format!(
                "Filtros por palavra-chave descartam currículos sem os termos exatos. Se você tem experiência, cite: {}.",
                list.join(", ")
            ));
        }
        if !missing_terms.is_empty() {
            let list: Vec<_> = missing_terms
                .iter()
                .take(6)
                .map(|m| format!("'{m}'"))
                .collect();
            suggestions.push(format!(
                "Use o vocabulário da própria vaga quando fizer sentido: {}.",
                list.join(", ")
            ));
        }

        Ok(ScoreReport {
            total: (total * 10.0).round() / 10.0,
            breakdown,
            matched_keywords: matched,
            missing_keywords: missing,
            suggestions,
            engine_version: ID.into(),
        })
    }
}
