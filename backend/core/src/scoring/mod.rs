//! Pontuação de currículos contra vagas (motor heurístico `heuristic-v1`).
//!
//! Simula como motores ATS pontuariam um currículo para uma vaga específica.
//! Critérios e pesos (ver CONTRACT.md):
//!
//! | key | peso |
//! |---|---|
//! | `hard_skills` | 0.40 |
//! | `semantic` | 0.20 |
//! | `experience` | 0.15 |
//! | `seniority_title` | 0.10 |
//! | `education_lang` | 0.05 |
//! | `ats_format` | 0.10 |
//!
//! A pontuação é determinística (sem aleatoriedade); apenas a data atual é usada
//! para resolver períodos como "2020 - atual" (use [`score_at`] para fixá-la).

pub mod experience;
pub mod job;
pub mod sections;
pub mod skills;
pub mod tfidf;

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

pub use job::{analyze_job, EducationLevel, JobProfile, JobSkill, KeyTerm, LanguageReq, Seniority};
pub use sections::SectionKind;
pub use skills::SkillCategory;

use crate::text::{tokens, NgramSet};
use job::{detect_seniority, LANGUAGES};

/// Versão do motor de pontuação.
pub const ENGINE_VERSION: &str = "heuristic-v1";

/// Relatório de pontuação de um currículo para uma vaga.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ScoreReport {
    /// 0..=100
    pub total: f64,
    pub breakdown: Vec<CriterionScore>,
    pub matched_keywords: Vec<String>,
    /// palavras importantes da vaga ausentes
    pub missing_keywords: Vec<String>,
    /// dicas em PT-BR para melhorar o currículo
    pub suggestions: Vec<String>,
    pub engine_version: String,
}

/// Pontuação de um critério.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CriterionScore {
    pub key: String,
    pub label: String,
    pub weight: f64,
    /// 0..=100
    pub score: f64,
    pub detail: String,
}

pub const W_HARD_SKILLS: f64 = 0.40;
pub const W_SEMANTIC: f64 = 0.20;
pub const W_EXPERIENCE: f64 = 0.15;
pub const W_SENIORITY_TITLE: f64 = 0.10;
pub const W_EDUCATION_LANG: f64 = 0.05;
pub const W_ATS_FORMAT: f64 = 0.10;

/// Pontua o currículo (texto já anonimizado) contra a vaga, usando a data atual.
pub fn score(job: &JobProfile, resume_text: &str) -> ScoreReport {
    let (y, m) = today();
    score_at(job, resume_text, y, m)
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn clamp100(x: f64) -> f64 {
    if x.is_finite() {
        x.clamp(0.0, 100.0)
    } else {
        0.0
    }
}

fn quoted_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| format!("'{s}'"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Igual a [`score`], mas com a data "atual" informada (para testes reprodutíveis).
pub fn score_at(job: &JobProfile, resume_text: &str, now_year: i32, now_month: u32) -> ScoreReport {
    let ng = NgramSet::from_text(resume_text);
    let resume_stems: HashSet<String> = tfidf::content_stems(resume_text).into_iter().collect();
    let fmt = sections::analyze(resume_text);
    let mut suggestions: Vec<String> = Vec::new();
    let mut matched_keywords: Vec<String> = Vec::new();
    let mut missing_keywords: Vec<String> = Vec::new();

    // ------------------------------------------------------------ hard skills
    let mut missing_required: Vec<String> = Vec::new();
    let mut missing_nice: Vec<String> = Vec::new();
    let (mut w_total, mut w_found) = (0.0, 0.0);
    let (mut n_req, mut n_req_found, mut n_found) = (0, 0, 0);
    for s in &job.skills {
        let present = match skills::by_name(&s.name) {
            Some(def) => skills::present(def, &ng, resume_text),
            None => ng.has_phrase(&s.name),
        };
        w_total += s.weight;
        if s.required {
            n_req += 1;
        }
        if present {
            w_found += s.weight;
            n_found += 1;
            if s.required {
                n_req_found += 1;
            }
            matched_keywords.push(s.name.clone());
        } else if s.required {
            missing_required.push(s.name.clone());
        } else {
            missing_nice.push(s.name.clone());
        }
    }
    missing_keywords.extend(missing_required.iter().cloned());
    missing_keywords.extend(missing_nice.iter().cloned());

    let mut missing_terms: Vec<String> = Vec::new();
    let (mut t_total, mut t_found) = (0.0, 0.0);
    for t in &job.important_terms {
        t_total += t.weight;
        if resume_stems.contains(&t.stem) {
            t_found += t.weight;
            matched_keywords.push(t.term.clone());
        } else {
            missing_terms.push(t.term.clone());
        }
    }
    missing_keywords.extend(missing_terms.iter().cloned());
    let term_cov = if t_total > 0.0 {
        t_found / t_total
    } else {
        0.0
    };

    let (hard_score, hard_detail) = if w_total > 0.0 {
        let skill_cov = w_found / w_total;
        let s = if t_total > 0.0 {
            0.85 * skill_cov + 0.15 * term_cov
        } else {
            skill_cov
        };
        let mut d = format!("{n_found} de {} skills encontradas", job.skills.len());
        if n_req > 0 && n_req != job.skills.len() {
            d.push_str(&format!(" (obrigatórias: {n_req_found} de {n_req})"));
        }
        (s * 100.0, d)
    } else if t_total > 0.0 {
        let found = job.important_terms.len() - missing_terms.len();
        (
            term_cov * 100.0,
            format!(
                "Nenhuma skill técnica reconhecida na vaga; {found} de {} palavras-chave encontradas",
                job.important_terms.len()
            ),
        )
    } else {
        (
            50.0,
            "Vaga sem skills ou palavras-chave reconhecíveis".to_string(),
        )
    };

    // ------------------------------------------------------------ semântica
    let cos = tfidf::cosine(
        &job.term_freqs,
        &job.line_doc_freq,
        job.line_count,
        resume_text,
    );
    let semantic_score = (cos / 0.45).min(1.0) * 100.0;
    let semantic_detail = format!(
        "Similaridade de conteúdo com a vaga (TF-IDF/cosseno): {:.0}%",
        cos * 100.0
    );

    // ------------------------------------------------------------ experiência
    let years_found = experience::estimate_years(resume_text, now_year, now_month);
    let years_req = job
        .years_required
        .or_else(|| job.seniority.map(|s| s.typical_years()));
    let has_exp_section = fmt.sections.contains(&SectionKind::Experience);
    let (exp_score, exp_detail) = match years_req {
        None => {
            if years_found > 0.0 {
                (
                    100.0,
                    format!("Vaga não especifica tempo; ~{years_found:.1} anos detectados"),
                )
            } else if has_exp_section {
                (
                    75.0,
                    "Vaga não especifica tempo; períodos de experiência não identificados"
                        .to_string(),
                )
            } else {
                (
                    50.0,
                    "Vaga não especifica tempo; nenhuma experiência identificada".to_string(),
                )
            }
        }
        Some(r) if r <= 0.0 => (100.0, "Vaga não exige experiência prévia".to_string()),
        Some(r) => {
            let source = if job.years_required.is_some() {
                "Vaga pede"
            } else {
                "Nível da vaga sugere"
            };
            if years_found <= 0.0 {
                let s = if has_exp_section { 15.0 } else { 0.0 };
                (
                    s,
                    format!("{source} {r:.0}+ anos; nenhum período de experiência identificado"),
                )
            } else {
                let ratio = (years_found / r).min(1.0);
                (
                    ratio.powf(0.8) * 100.0,
                    format!("{source} {r:.0}+ anos; ~{years_found:.1} anos detectados"),
                )
            }
        }
    };
    if let Some(r) = years_req.filter(|r| *r > 0.0) {
        if years_found <= 0.0 {
            suggestions.push(
                "Não identificamos seu tempo de experiência: informe os períodos de cada experiência no formato 'mm/aaaa – mm/aaaa' (ou 'atual').".to_string(),
            );
        } else if years_found + 0.5 < r {
            suggestions.push(format!(
                "A vaga pede cerca de {r:.0} anos de experiência e identificamos ~{years_found:.1}. Deixe claras as datas de início e fim de cada experiência e inclua experiências relevantes (freelas, projetos, estágios)."
            ));
        }
    }

    // ------------------------------------------------------------ título / senioridade
    let title_found = job
        .title_terms
        .iter()
        .filter(|t| resume_stems.contains(*t))
        .count();
    let title_part = if job.title_terms.is_empty() {
        100.0
    } else {
        title_found as f64 / job.title_terms.len() as f64 * 100.0
    };
    let mentioned = detect_seniority(resume_text, false);
    let by_years =
        (years_found > 0.0).then(|| Seniority::from_years(years_found).min(Seniority::Senior));
    let resume_level = match (mentioned, by_years) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    };
    let sen_part = match (job.seniority, resume_level) {
        (None, _) => 100.0,
        (Some(_), None) => 50.0,
        (Some(j), Some(r)) => match r.rank() - j.rank() {
            0 => 100.0,
            1 => 85.0,
            d if d >= 2 => 70.0,
            -1 => 55.0,
            -2 => 25.0,
            _ => 10.0,
        },
    };
    let seniority_score = 0.5 * title_part + 0.5 * sen_part;
    let seniority_detail = {
        let mut d = if job.title_terms.is_empty() {
            "Título da vaga sem termos específicos".to_string()
        } else {
            format!(
                "{title_found} de {} termos do título da vaga presentes",
                job.title_terms.len()
            )
        };
        match (job.seniority, resume_level) {
            (Some(j), Some(r)) => d.push_str(&format!(
                "; senioridade detectada: {} (vaga: {})",
                r.label(),
                j.label()
            )),
            (Some(j), None) => d.push_str(&format!(
                "; senioridade não identificada (vaga: {})",
                j.label()
            )),
            (None, Some(r)) => d.push_str(&format!("; senioridade detectada: {}", r.label())),
            (None, None) => {}
        }
        d
    };
    if title_part < 100.0 && !job.title.is_empty() {
        suggestions.push(format!(
            "Inclua o título da vaga ('{}') no seu resumo ou cabeçalho, se for coerente com sua trajetória.",
            job.title
        ));
    }
    if let (Some(j), Some(r)) = (job.seniority, resume_level) {
        if r < j && j >= Seniority::Senior {
            suggestions.push(format!(
                "A vaga é para nível {}: evidencie responsabilidades compatíveis (decisões de arquitetura, liderança técnica, mentoria, impacto no negócio).",
                j.label()
            ));
        }
    }

    // ------------------------------------------------------------ formação / idiomas
    let english_text = is_mostly_english(resume_text);
    let mut comps: Vec<(f64, f64)> = Vec::new(); // (peso, 0..1)
    let mut edu_ok = 0;
    let mut missing_edu: Vec<&str> = Vec::new();
    for lvl in &job.education {
        let ok = match lvl {
            EducationLevel::Superior => [
                EducationLevel::Superior,
                EducationLevel::PosGraduacao,
                EducationLevel::Doutorado,
            ]
            .iter()
            .any(|l| ng.has_any(l.phrases(true))),
            EducationLevel::PosGraduacao => {
                [EducationLevel::PosGraduacao, EducationLevel::Doutorado]
                    .iter()
                    .any(|l| ng.has_any(l.phrases(true)))
            }
            EducationLevel::Doutorado => ng.has_any(lvl.phrases(true)),
            EducationLevel::Certificacao => {
                ng.has_any(lvl.phrases(true)) || fmt.sections.contains(&SectionKind::Certifications)
            }
        };
        if ok {
            edu_ok += 1;
        } else {
            missing_edu.push(lvl.label());
        }
        comps.push((1.0, if ok { 1.0 } else { 0.0 }));
    }
    let mut lang_ok = 0;
    let mut missing_langs: Vec<String> = Vec::new();
    for l in &job.languages {
        let phrases = LANGUAGES
            .iter()
            .find(|(n, _)| *n == l.language)
            .map(|(_, p)| *p)
            .unwrap_or(&[]);
        let ok = ng.has_any(phrases)
            || (l.language == "Inglês" && english_text)
            || (l.language == "Português" && !english_text);
        if ok {
            lang_ok += 1;
        } else {
            missing_langs.push(l.language.clone());
        }
        comps.push((
            if l.required { 1.0 } else { 0.5 },
            if ok { 1.0 } else { 0.0 },
        ));
    }
    let (edu_score, edu_detail) = if comps.is_empty() {
        let has_edu = fmt.sections.contains(&SectionKind::Education)
            || ng.has_any(EducationLevel::Superior.phrases(true));
        (
            if has_edu { 100.0 } else { 60.0 },
            if has_edu {
                "Vaga sem exigências de formação/idiomas; formação informada".to_string()
            } else {
                "Vaga sem exigências de formação/idiomas; formação não informada".to_string()
            },
        )
    } else {
        let tw: f64 = comps.iter().map(|c| c.0).sum();
        let s: f64 = comps.iter().map(|c| c.0 * c.1).sum::<f64>() / tw;
        let mut parts = Vec::new();
        if !job.education.is_empty() {
            parts.push(format!("formação {edu_ok} de {}", job.education.len()));
        }
        if !job.languages.is_empty() {
            let names = job
                .languages
                .iter()
                .map(|l| l.language.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!(
                "idiomas {lang_ok} de {} ({names})",
                job.languages.len()
            ));
        }
        (
            s * 100.0,
            format!("Requisitos atendidos: {}", parts.join("; ")),
        )
    };
    for lang in &missing_langs {
        suggestions.push(format!(
            "Mencione seu nível de {lang} (ex.: '{lang} avançado') — a vaga pede esse idioma."
        ));
    }
    if !missing_edu.is_empty() {
        suggestions.push(format!(
            "Informe claramente sua formação ({}) — a vaga menciona esse requisito.",
            missing_edu.join(", ")
        ));
    }

    // ------------------------------------------------------------ formato ATS
    let mut ats = 0.0;
    let has = |k: SectionKind| fmt.sections.contains(&k);
    for (k, pts) in [
        (SectionKind::Experience, 15.0),
        (SectionKind::Education, 15.0),
        (SectionKind::Skills, 15.0),
        (SectionKind::Summary, 10.0),
    ] {
        if has(k) {
            ats += pts;
        }
    }
    ats += match fmt.word_count {
        250..=1200 => 20.0,
        150..=249 | 1201..=2000 => 12.0,
        60..=149 => 6.0,
        _ => 0.0,
    };
    ats += match fmt.bullet_lines {
        0 => 0.0,
        1..=2 => 5.0,
        _ => 10.0,
    };
    ats += if fmt.symbol_ratio < 0.03 {
        15.0
    } else if fmt.symbol_ratio < 0.08 {
        8.0
    } else {
        0.0
    };
    let ats_detail = {
        let secs = if fmt.sections.is_empty() {
            "nenhuma seção reconhecida".to_string()
        } else {
            format!(
                "seções: {}",
                fmt.sections
                    .iter()
                    .map(|s| s.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!(
            "{secs}; {} palavras; {} tópicos (bullets)",
            fmt.word_count, fmt.bullet_lines
        )
    };

    // ------------------------------------------------------------ sugestões de skills (prioridade)
    let mut skill_sugs: Vec<String> = missing_required
        .iter()
        .take(5)
        .map(|s| {
            format!("Inclua explicitamente '{s}' se você tiver experiência — é requisito da vaga.")
        })
        .collect();
    if missing_required.len() > 5 {
        skill_sugs.push(format!(
            "Outras skills pedidas e não encontradas no currículo: {}.",
            quoted_list(&missing_required[5..])
        ));
    }
    if !missing_nice.is_empty() {
        skill_sugs.push(format!(
            "Se tiver, mencione também os diferenciais da vaga: {}.",
            quoted_list(&missing_nice)
        ));
    }
    if semantic_score < 60.0 && !missing_terms.is_empty() {
        let few: Vec<String> = missing_terms.iter().take(5).cloned().collect();
        skill_sugs.push(format!(
            "Use a mesma terminologia da vaga ao descrever suas experiências, como: {}.",
            quoted_list(&few)
        ));
    }
    suggestions.splice(0..0, skill_sugs);

    for (k, msg) in [
        (SectionKind::Experience, "Adicione uma seção 'Experiência Profissional' clara, com cargo, empresa, período e principais entregas."),
        (SectionKind::Skills, "Adicione uma seção 'Habilidades' listando as tecnologias e ferramentas que você domina."),
        (SectionKind::Education, "Adicione uma seção 'Formação' com curso, instituição e ano de conclusão."),
        (SectionKind::Summary, "Inclua um breve 'Resumo' no topo com seu cargo-alvo e suas principais competências."),
    ] {
        if !has(k) {
            suggestions.push(msg.to_string());
        }
    }
    if fmt.quantified_lines < 2 {
        suggestions.push(
            "Quantifique resultados (ex.: 'reduzi o tempo de deploy em 40%', 'atendi 20 mil usuários/mês').".to_string(),
        );
    }
    if fmt.bullet_lines == 0 {
        suggestions.push(
            "Use tópicos (bullets) para descrever atividades e conquistas — facilita a leitura por ATS e recrutadores.".to_string(),
        );
    }
    if fmt.word_count < 250 {
        suggestions.push(format!(
            "Seu currículo está curto ({} palavras): detalhe experiências, tecnologias usadas e resultados.",
            fmt.word_count
        ));
    } else if fmt.word_count > 1200 {
        suggestions.push(format!(
            "Seu currículo está longo ({} palavras): priorize o que é relevante para esta vaga.",
            fmt.word_count
        ));
    }
    if fmt.symbol_ratio >= 0.03 {
        suggestions.push(
            "Evite ícones, tabelas, gráficos de nível e caracteres especiais — muitos ATS não conseguem lê-los.".to_string(),
        );
    }
    if suggestions.is_empty() {
        suggestions.push(
            "Seu currículo está bem alinhado com a vaga. Revise para mantê-lo objetivo e atualizado.".to_string(),
        );
    }

    // ------------------------------------------------------------ total
    let breakdown = vec![
        CriterionScore {
            key: "hard_skills".into(),
            label: "Habilidades técnicas".into(),
            weight: W_HARD_SKILLS,
            score: round1(clamp100(hard_score)),
            detail: hard_detail,
        },
        CriterionScore {
            key: "semantic".into(),
            label: "Similaridade com a vaga".into(),
            weight: W_SEMANTIC,
            score: round1(clamp100(semantic_score)),
            detail: semantic_detail,
        },
        CriterionScore {
            key: "experience".into(),
            label: "Tempo de experiência".into(),
            weight: W_EXPERIENCE,
            score: round1(clamp100(exp_score)),
            detail: exp_detail,
        },
        CriterionScore {
            key: "seniority_title".into(),
            label: "Cargo e senioridade".into(),
            weight: W_SENIORITY_TITLE,
            score: round1(clamp100(seniority_score)),
            detail: seniority_detail,
        },
        CriterionScore {
            key: "education_lang".into(),
            label: "Formação e idiomas".into(),
            weight: W_EDUCATION_LANG,
            score: round1(clamp100(edu_score)),
            detail: edu_detail,
        },
        CriterionScore {
            key: "ats_format".into(),
            label: "Formato para ATS".into(),
            weight: W_ATS_FORMAT,
            score: round1(clamp100(ats)),
            detail: ats_detail,
        },
    ];
    let total = round1(clamp100(breakdown.iter().map(|c| c.weight * c.score).sum()));

    dedup_ci(&mut matched_keywords);
    dedup_ci(&mut missing_keywords);

    ScoreReport {
        total,
        breakdown,
        matched_keywords,
        missing_keywords,
        suggestions,
        engine_version: ENGINE_VERSION.to_string(),
    }
}

fn dedup_ci(v: &mut Vec<String>) {
    let mut seen = HashSet::new();
    v.retain(|s| seen.insert(s.to_lowercase()));
}

/// Heurística simples: o texto tem mais stopwords em inglês do que em português?
pub fn is_mostly_english(text: &str) -> bool {
    const EN: &[&str] = &[
        "the", "and", "with", "of", "to", "in", "for", "on", "at", "my", "i",
    ];
    const PT: &[&str] = &[
        "de", "e", "com", "em", "para", "que", "da", "do", "na", "no", "os", "as",
    ];
    let (mut en, mut pt) = (0, 0);
    for t in tokens(text) {
        if EN.contains(&t.as_str()) {
            en += 1;
        }
        if PT.contains(&t.as_str()) {
            pt += 1;
        }
    }
    en > pt && en >= 5
}

/// (ano, mês) atuais em UTC, sem dependências externas.
fn today() -> (i32, u32) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y as i32, m as u32)
}

#[cfg(test)]
mod tests;
